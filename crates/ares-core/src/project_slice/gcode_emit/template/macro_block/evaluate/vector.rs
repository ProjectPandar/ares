//! Vector output variables of a macro block (`PlaceholderParser.cpp:1428-1490, 1602-1818`):
//! initializer lists, `repeat`, copies of a naked vector and assigned elements.

use super::super::super::super::expression::{
    Operand, PathError, assigned_value, element_count, element_value, evaluate_operand,
};
use super::super::super::super::value::Value;
use super::super::rhs::Reference;
use super::super::{Scope, VectorKind};
use super::{Block, assignment_index};

const VECTOR_TO_SCALAR: &str = "Cannot assign a vector value to a scalar variable.";
const NIL_ELEMENTS: &str =
    "Some elements of the right hand side vector variable of optional values are undefined (nil)";

impl Block<'_> {
    /// The vector type of `name`: as declared in this render, or of the system vector.
    pub(super) fn vector_kind(&self, name: &str) -> Option<VectorKind> {
        match self.journal.iter().find(|(declared, ..)| declared == name) {
            Some((.., kind)) => *kind,
            None => (self.config.get(name)).and_then(|value| VectorKind::of_config(name, value)),
        }
    }

    /// `vector_variable_assign_initializer_list` (`:1651-1694`): a one-element list assigns a
    /// scalar, and a list replaces a vector whole, whether or not the target is indexed.
    pub(super) fn assign_list(
        &mut self,
        name: &str,
        kind: Option<VectorKind>,
        elements: &[&str],
    ) -> Result<(), PathError> {
        let operands = self.operands(elements)?;
        self.writable(name)?;
        let Some(kind) = kind else {
            let [(operand, element)] = &operands[..] else {
                return Err(self.failure(name, VECTOR_TO_SCALAR));
            };
            let variable = self.config.get(name).cloned().unwrap_or(Value::Nil);
            let value =
                assigned_value(&variable, operand).map_err(|error| self.located(error, element))?;
            self.config.assign(name, None, value);
            return Ok(());
        };
        let values = self.fill(name, kind, &operands)?;
        self.store(name, values);
        Ok(())
    }

    /// `vector_variable_assign_array` (`:1628-1640`) and the assignment of
    /// `vector_variable_new_from_array` (`:1610-1614`).
    pub(super) fn assign_repeat(
        &mut self,
        name: &str,
        kind: Option<VectorKind>,
        (count, value): (&str, &str),
    ) -> Result<(), PathError> {
        let count = (count, self.operand(count)?);
        let value = (value, self.operand(value)?);
        self.writable(name)?;
        let Some(kind) = kind else {
            return Err(self.failure(value.0, VECTOR_TO_SCALAR));
        };
        let values = self.repeated(name, kind, &count, &value)?;
        self.store(name, values);
        Ok(())
    }

    /// `copy_vector_variable_to_vector_variable` (`:1757-1786`).
    pub(super) fn copy(
        &mut self,
        name: &str,
        kind: VectorKind,
        reference: &Reference<'_>,
    ) -> Result<(), PathError> {
        let Some(source) = self.config.get(reference.name) else {
            return Err(self.failure(reference.name, "Not a variable name"));
        };
        let values: Vec<Value> = source.iter_list().cloned().collect();
        self.writable(name)?;
        let source_kind = (!reference.indexed)
            .then(|| self.vector_kind(reference.name))
            .flatten();
        let Some(source_kind) = source_kind else {
            return Err(self.failure(name, "Cannot assign scalar to a vector"));
        };
        if source_kind.is_nil(&values) {
            return Err(self.failure(reference.name, NIL_ELEMENTS));
        }
        if source_kind != kind {
            return Err(self.failure(name, kind.mismatch()));
        }
        self.check_rest(reference)?;
        self.store(name, values);
        Ok(())
    }

    /// `vector_variable_new_from_copy` (`:1788-1818`): the type and elements a new variable
    /// copies from a naked vector, or none for an indexed or scalar reference.
    pub(super) fn duplicate(
        &self,
        reference: &Reference<'_>,
    ) -> Result<Option<(VectorKind, Vec<Value>)>, PathError> {
        let Some(source) = self.config.get(reference.name) else {
            return Err(self.failure(reference.name, "Not a variable name"));
        };
        let Some(kind) = (!reference.indexed)
            .then(|| self.vector_kind(reference.name))
            .flatten()
        else {
            return Ok(None);
        };
        let values: Vec<Value> = source.iter_list().cloned().collect();
        if kind.is_nil(&values) {
            return Err(self.failure(reference.name, NIL_ELEMENTS));
        }
        let Some(kind) = kind.duplicate() else {
            return Err(self.failure(
                reference.name,
                "Duplicating this type of vector variable is not supported",
            ));
        };
        self.check_rest(reference)?;
        Ok(Some((kind, values)))
    }

    /// `vector_variable_element_assign_scalar` (`:1428-1463`).
    pub(super) fn assign_element(
        &mut self,
        name: &str,
        kind: VectorKind,
        index: Option<&str>,
        rhs: &str,
    ) -> Result<(), PathError> {
        let index =
            assignment_index(name, index, self.config, self.filament).map_err(PathError::Other)?;
        let operand = self.operand(rhs)?;
        self.writable(name)?;
        let Some(index) = index else {
            return Err(self.failure(
                name,
                "Referencing an output vector variable when scalar is expected",
            ));
        };
        let size = self
            .config
            .get(name)
            .map_or(0, |value| value.iter_list().len());
        if size == 0 {
            return Err(self.failure(name, "Indexing an empty vector variable"));
        }
        if index >= size {
            return Err(self.failure(name, "Index out of range"));
        }
        let value = assigned_value(&self.prototype(name, kind)?, &operand)
            .map_err(|error| self.located(error, rhs))?;
        self.config.assign(name, Some(index), value);
        Ok(())
    }

    /// `vector_variable_new_from_initializer_list` (`:1696-1745`): the vector type is deduced
    /// from the elements.
    pub(super) fn new_list(
        &mut self,
        name: &str,
        scope: Scope,
        elements: &[&str],
    ) -> Result<(), PathError> {
        let operands = self.operands(elements)?;
        let Some(first) = elements.first() else {
            return Err(self.failure(
                name,
                "Cannot create vector variable from an empty initializer list, because its type \
                 cannot be deduced.",
            ));
        };
        let has = |matches: fn(&Value) -> bool| {
            operands
                .iter()
                .any(|(operand, _)| matches(operand.value.scalar()))
        };
        let numeric = has(|value| matches!(value, Value::Integer(_) | Value::Number(_)));
        let kind = if has(|value| matches!(value, Value::String(_))) {
            VectorKind::Strings
        } else if has(|value| matches!(value, Value::Bool(_) | Value::OptionBool(_))) {
            if numeric {
                return Err(self.failure(
                    first,
                    "Right side is not valid: Mixing numeric and boolean types.",
                ));
            }
            VectorKind::Bools
        } else if has(|value| matches!(value, Value::Number(_))) {
            VectorKind::Floats
        } else {
            VectorKind::Ints
        };
        let values = self.fill(name, kind, &operands)?;
        self.store_new(name, (scope, Some(kind)), values);
        Ok(())
    }

    /// `vector_variable_new_from_array` (`:1615-1624`): the vector takes the type of the value.
    pub(super) fn new_repeat(
        &mut self,
        name: &str,
        scope: Scope,
        (count, value): (&str, &str),
    ) -> Result<(), PathError> {
        let count = (count, self.operand(count)?);
        let value = (value, self.operand(value)?);
        let kind = VectorKind::of_element(&value.1.value);
        let values = self.repeated(name, kind, &count, &value)?;
        self.store_new(name, (scope, Some(kind)), values);
        Ok(())
    }

    /// `evaluate_count` (`:1387-1394`), then `count` copies of the value in the vector type
    /// (`vector_variable_assign_expr_with_count`, `:1465-1490`).
    fn repeated(
        &self,
        name: &str,
        kind: VectorKind,
        (count, count_operand): &(&str, Operand),
        (value, value_operand): &(&str, Operand),
    ) -> Result<Vec<Value>, PathError> {
        let count = element_count(count_operand).map_err(|error| self.located(error, count))?;
        let element = element_value(&self.prototype(name, kind)?, value_operand)
            .map_err(|error| self.located(error, value))?;
        Ok(vec![element; count])
    }

    /// The elements of a list in the vector type (`fill_vector_from_initializer_list`,
    /// `:1642-1649`).
    fn fill(
        &self,
        name: &str,
        kind: VectorKind,
        operands: &[(Operand, &str)],
    ) -> Result<Vec<Value>, PathError> {
        let prototype = self.prototype(name, kind)?;
        operands
            .iter()
            .map(|(operand, element)| {
                element_value(&prototype, operand).map_err(|error| self.located(error, element))
            })
            .collect()
    }

    fn prototype(&self, name: &str, kind: VectorKind) -> Result<Value, PathError> {
        kind.prototype()
            .ok_or_else(|| self.failure(name, "Unsupported output vector variable type"))
    }

    fn operands<'b>(&self, elements: &[&'b str]) -> Result<Vec<(Operand, &'b str)>, PathError> {
        elements
            .iter()
            .map(|element| Ok((self.operand(element)?, *element)))
            .collect()
    }

    fn operand(&self, part: &str) -> Result<Operand, PathError> {
        evaluate_operand(part, self.config, self.filament)
            .map_err(|error| self.located(error, part))
    }

    /// A reference copy ends the right-hand side.
    fn check_rest(&self, reference: &Reference<'_>) -> Result<(), PathError> {
        if reference.rest.is_empty() {
            Ok(())
        } else {
            Err(PathError::Other(format!(
                "unexpected {:?} in macro block",
                reference.rest
            )))
        }
    }

    pub(super) fn store_new(
        &mut self,
        name: &str,
        (scope, kind): (Scope, Option<VectorKind>),
        values: Vec<Value>,
    ) {
        self.journal.push((name.to_owned(), scope, kind));
        self.store(name, values);
    }

    /// Replaces a vector whole, with the spread elements of the old one.
    fn store(&mut self, name: &str, values: Vec<Value>) {
        self.config.remove(name);
        self.config.insert(name, Value::List(values));
    }
}
