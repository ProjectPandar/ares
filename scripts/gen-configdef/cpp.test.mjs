import assert from 'node:assert/strict';
import test from 'node:test';

import { decodeStringLiterals } from './cpp.mjs';

test('decodes only the msgid from L_CONTEXT', () => {
    assert.equal(decodeStringLiterals('L_CONTEXT("s", "second")'), 's');
    assert.equal(decodeStringLiterals('L_CONTEXT("X", "Axis")'), 'X');
    assert.equal(decodeStringLiterals('  L_CONTEXT (u8"X" " axis", "Axis (direction)")'), 'X axis');
});

test('preserves ordinary adjacent C++ string literals', () => {
    assert.equal(decodeStringLiterals('"first" " second"'), 'first second');
    assert.equal(decodeStringLiterals('L("first" " second")'), 'first second');
});

test('decodes escaped context msgids without consuming context', () => {
    assert.equal(decodeStringLiterals('L_CONTEXT("wall\\n", "tooltip, context")'), 'wall\n');
});
