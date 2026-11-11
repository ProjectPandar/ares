//! Statement evaluation of one macro block (`PlaceholderParser.cpp:2157-2186`): expression
//! statements, assignments, `local`/`global` declarations and inline `if` blocks.

use super::super::super::expression::{
    ParseFailure, ParsePath, PathError, evaluate, evaluate_assigned, evaluate_condition,
    evaluate_path, to_string,
};
use super::super::super::value::{Config, Value};
use super::super::offset_in;
use super::rhs::{Reference, Rhs};
use super::{Arm, Journal, Scope, Statement};

mod vector;

/// The writable system output variables (`GCode.cpp:2189-2209`).
const SYSTEM_OUTPUTS: [&str; 4] = ["position", "e_position", "e_retracted", "e_restart_extra"];

/// Evaluates an assignment's optional index expression to a valid
/// vector index (`assignment_statement`, `:2170-2174`).
fn assignment_index(
    name: &str,
    index: Option<&str>,
    config: &Config,
    filament: usize,
) -> Result<Option<usize>, String> {
    let Some(index) = index else {
        return Ok(None);
    };
    match evaluate(index, config, filament)?.as_number() {
        Some(number) if number >= 0.0 => Ok(Some(number as usize)),
        _ => Err(format!("invalid assignment index in {name}[{index}]")),
    }
}

/// The evaluation state of one block: its body, for the offsets of failures, the config
/// mutated by assignments and the journal of `local`/`global` output variables.
pub(super) struct Block<'a> {
    pub(super) body: &'a str,
    pub(super) config: &'a mut Config,
    pub(super) filament: usize,
    pub(super) journal: &'a mut Journal,
}

impl Block<'_> {
    /// Evaluates a `;`-separated statement list, appending rendered
    /// expression statements to `output` (`PlaceholderParser.cpp:2157-2186`).
    pub(super) fn statements(
        &mut self,
        statements: &[Statement<'_>],
        output: &mut String,
    ) -> Result<(), PathError> {
        for statement in statements {
            match statement {
                Statement::Expression(expression) => {
                    output.push_str(&to_string(&self.value(expression)?));
                }
                Statement::Assignment { name, index, rhs } => self.assign(name, *index, rhs)?,
                Statement::Declare { name, rhs, scope } => self.declare(name, rhs, *scope)?,
                Statement::If(arms) => self.branch(arms, output)?,
            }
        }
        Ok(())
    }

    /// `assignment_statement` (`:2435-2448`): the target resolves before the right-hand side
    /// (`resolve_variable`, `:1123-1145`) and is checked writable after it (`check_writable`,
    /// `:1377-1380`). An initializer list and `repeat` fill a vector, a naked vector reference
    /// copies into a naked vector, and any other expression assigns a scalar or an element.
    fn assign(&mut self, name: &str, index: Option<&str>, rhs: &str) -> Result<(), PathError> {
        if self.config.get(name).is_none() {
            return Err(self.failure(name, "Not a variable name"));
        }
        let kind = self.vector_kind(name);
        match (Rhs::parse(rhs), kind) {
            (Rhs::List(elements), kind) => self.assign_list(name, kind, &elements),
            (Rhs::Repeat(count, value), kind) => self.assign_repeat(name, kind, (count, value)),
            (Rhs::Expression, Some(kind)) => match (index, Reference::parse(rhs)) {
                (None, Some(reference)) => self.copy(name, kind, &reference),
                _ => self.assign_element(name, kind, index, rhs),
            },
            (Rhs::Expression, None) => self.assign_scalar(name, index, rhs),
        }
    }

    /// `scalar_variable_assign_scalar` (`:1396-1426`): a declared variable keeps its type.
    fn assign_scalar(
        &mut self,
        name: &str,
        index: Option<&str>,
        rhs: &str,
    ) -> Result<(), PathError> {
        let value = match self.config.get(name) {
            Some(variable) if self.scope(name).is_some() => self.assigned(rhs, variable)?,
            _ => {
                let value = self.value(rhs)?;
                self.writable(name)?;
                value
            }
        };
        let index =
            assignment_index(name, index, self.config, self.filament).map_err(PathError::Other)?;
        self.config.assign(name, index, value);
        Ok(())
    }

    /// `check_writable` (`:1377-1380`): only `local`, `global` and system output variables.
    fn writable(&self, name: &str) -> Result<(), PathError> {
        if self.scope(name).is_some() || SYSTEM_OUTPUTS.contains(&name) {
            Ok(())
        } else {
            Err(self.failure(name, "Cannot modify a read-only variable"))
        }
    }

    /// `new_old_variable` (`:1529-1564`) checks the name before the right-hand side. A name
    /// declared before in the same scope is assigned in its type
    /// (`scalar_variable_new_from_scalar_expression`, `:1577-1600`); a new name becomes an
    /// output variable dropped after the render.
    fn declare(&mut self, name: &str, rhs: &str, scope: Scope) -> Result<(), PathError> {
        let declared = self.scope(name);
        let conflict = match (declared, scope) {
            (None, _) if SYSTEM_OUTPUTS.contains(&name) && self.config.get(name).is_some() => {
                Some("Symbol is already defined as system output variable")
            }
            (None, _) if self.config.get(name).is_some() => {
                Some("Symbol is already defined in read-only system dictionary")
            }
            (Some(Scope::Local), Scope::Global) => {
                Some("Variable name already defined in local scope")
            }
            (Some(Scope::Global), Scope::Local) => {
                Some("Variable name already defined in global scope")
            }
            _ => None,
        };
        if let Some(message) = conflict {
            return Err(self.failure(name, message));
        }
        if declared.is_some() {
            return self.redeclare(name, rhs);
        }
        match Rhs::parse(rhs) {
            Rhs::List(elements) => self.new_list(name, scope, &elements),
            Rhs::Repeat(count, value) => self.new_repeat(name, scope, (count, value)),
            Rhs::Expression => {
                if let Some(reference) = Reference::parse(rhs)
                    && let Some((kind, values)) = self.duplicate(&reference)?
                {
                    self.store_new(name, (scope, Some(kind)), values);
                    return Ok(());
                }
                let value = self.value(rhs)?;
                self.journal.push((name.to_owned(), scope, None));
                self.config.assign(name, None, value);
                Ok(())
            }
        }
    }

    /// A declaration of a name declared before in the same scope assigns it
    /// (`scalar_variable_new_from_scalar_expression`, `:1577-1600`,
    /// `vector_variable_new_from_initializer_list`, `vector_variable_new_from_copy` and
    /// `vector_variable_new_from_array`).
    fn redeclare(&mut self, name: &str, rhs: &str) -> Result<(), PathError> {
        let kind = self.vector_kind(name);
        match (Rhs::parse(rhs), kind) {
            (Rhs::List(elements), kind) => self.assign_list(name, kind, &elements),
            (Rhs::Repeat(count, value), kind) => self.assign_repeat(name, kind, (count, value)),
            (Rhs::Expression, Some(kind)) => match Reference::parse(rhs) {
                Some(reference) => self.copy(name, kind, &reference),
                None => {
                    self.value(rhs)?;
                    Err(self.failure(rhs, "Cannot assign a scalar value to a vector variable."))
                }
            },
            (Rhs::Expression, None) => {
                let value = match self.config.get(name) {
                    Some(variable) => self.assigned(rhs, variable)?,
                    None => self.value(rhs)?,
                };
                self.config.assign(name, None, value);
                Ok(())
            }
        }
    }

    /// The scope `name` was declared in during this render.
    fn scope(&self, name: &str) -> Option<Scope> {
        self.journal
            .iter()
            .find_map(|(declared, scope, _)| (declared == name).then_some(*scope))
    }

    /// A failure at `name`, a slice of the block body.
    fn failure(&self, name: &str, message: &str) -> PathError {
        PathError::Parse(ParseFailure {
            message: message.to_owned(),
            offset: offset_in(self.body, name),
        })
    }

    fn assigned(&self, rhs: &str, variable: &Value) -> Result<Value, PathError> {
        evaluate_assigned(rhs, self.config, self.filament, variable)
            .map_err(|error| self.located(error, rhs))
    }

    /// `if_else_output` (`:2341-2349`): every condition is evaluated in order, also after an
    /// arm was taken (`:2345`), and the first true arm runs when its condition is reached.
    fn branch(&mut self, arms: &[Arm<'_>], output: &mut String) -> Result<(), PathError> {
        let mut taken = false;
        for arm in arms {
            let holds = match arm.condition {
                None => true,
                Some(condition) => self.condition(condition)?,
            };
            if holds && !taken {
                taken = true;
                self.statements(&arm.statements, output)?;
            }
        }
        Ok(())
    }

    fn condition(&self, condition: &str) -> Result<bool, PathError> {
        evaluate_condition(condition, self.config, self.filament)
            .map_err(|error| self.located(error, condition))
    }

    fn value(&self, expression: &str) -> Result<Value, PathError> {
        evaluate_path(expression, self.config, ParsePath::Brace, self.filament)
            .map_err(|error| self.located(error, expression))
    }

    /// A failure of `part`, a slice of the block body, in block coordinates.
    fn located(&self, error: PathError, part: &str) -> PathError {
        error
            .in_expression(part)
            .shifted(offset_in(self.body, part))
    }
}
