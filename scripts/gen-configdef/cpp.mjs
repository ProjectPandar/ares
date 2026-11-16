export function stripComments(src) {
    // Remove // and /* */ comments outside of string literals.
    let out = '';
    let i = 0;
    const n = src.length;
    while (i < n) {
        const c = src[i];
        if (c === '"') {
            const start = i;
            i += 1;
            while (i < n && src[i] !== '"') {
                if (src[i] === '\\') i += 1;
                i += 1;
            }
            i += 1;
            out += src.slice(start, i);
        } else if (c === '/' && src[i + 1] === '/') {
            while (i < n && src[i] !== '\n') i += 1;
        } else if (c === '/' && src[i + 1] === '*') {
            i += 2;
            while (i < n && !(src[i] === '*' && src[i + 1] === '/')) {
                if (src[i] === '\n') out += '\n';
                i += 1;
            }
            i += 2;
        } else {
            out += c;
            i += 1;
        }
    }
    // Drop dead `#if 0` blocks and directive lines so they cannot glue onto
    // the following statement.
    out = out.replace(/#if\s+0[\s\S]*?#endif/g, '\n');
    out = out.replace(/^\s*#(include|define|if|ifdef|ifndef|elif|else|endif)\b[^\n]*/gm, '');
    return out;
}

// Decode a C++ string literal sequence such as `"abc" "def"` or u8"..." into
// the UTF-8 string. Adjacent literals concatenate.
export function decodeStringLiterals(text) {
    const contextCall = /^\s*L_CONTEXT\s*\(/.exec(text);
    if (contextCall) {
        const [args] = callArgs(text, contextCall[0].length - 1);
        [text] = splitArgs(args);
    }
    const re = /(?:u8)?"((?:[^"\\]|\\.)*)"/g;
    let m;
    let out = '';
    let found = false;
    while ((m = re.exec(text)) !== null) {
        found = true;
        out += decodeCStringBody(m[1]);
    }
    return found ? out : null;
}

function decodeCStringBody(body) {
    let out = '';
    let i = 0;
    while (i < body.length) {
        const c = body[i];
        if (c !== '\\') {
            out += c;
            i += 1;
            continue;
        }
        i += 1;
        const e = body[i];
        switch (e) {
            case 'n': out += '\n'; i += 1; break;
            case 'r': out += '\r'; i += 1; break;
            case 't': out += '\t'; i += 1; break;
            case '0': out += '\0'; i += 1; break;
            case '\\': out += '\\'; i += 1; break;
            case '"': out += '"'; i += 1; break;
            case "'": out += "'"; i += 1; break;
            case 'x': {
                const m = /^[0-9a-fA-F]{1,4}/.exec(body.slice(i + 1));
                out += String.fromCodePoint(parseInt(m[0], 16));
                i += 1 + m[0].length;
                break;
            }
            case 'u': {
                out += String.fromCodePoint(parseInt(body.slice(i + 1, i + 5), 16));
                i += 5;
                break;
            }
            case 'U': {
                out += String.fromCodePoint(parseInt(body.slice(i + 1, i + 9), 16));
                i += 9;
                break;
            }
            default:
                out += e;
                i += 1;
        }
    }
    return out;
}

// Extract the argument list of a call starting at index `open` (position of '(').
export function callArgs(src, open) {
    let depth = 0;
    let i = open;
    const start = open + 1;
    for (; i < src.length; i += 1) {
        const c = src[i];
        if (c === '"') {
            i += 1;
            while (i < src.length && src[i] !== '"') {
                if (src[i] === '\\') i += 1;
                i += 1;
            }
        } else if (c === '(') depth += 1;
        else if (c === ')') {
            depth -= 1;
            if (depth === 0) return [src.slice(start, i), i];
        }
    }
    throw new Error('unbalanced parens');
}

export function splitArgs(text) {
    const parts = [];
    let depth = 0;
    let cur = '';
    for (let i = 0; i < text.length; i += 1) {
        const c = text[i];
        if (c === '"') {
            const start = i;
            i += 1;
            while (i < text.length && text[i] !== '"') {
                if (text[i] === '\\') i += 1;
                i += 1;
            }
            cur += text.slice(start, i + 1);
            continue;
        }
        if (c === '(' || c === '{' || c === '[') depth += 1;
        if (c === ')' || c === '}' || c === ']') depth -= 1;
        if (c === ',' && depth === 0) {
            parts.push(cur.trim());
            cur = '';
        } else {
            cur += c;
        }
    }
    if (cur.trim() !== '') parts.push(cur.trim());
    return parts;
}

// Extract a function body by name.
export function fnBody(src, name) {
    const start = src.indexOf(`PrintConfigDef::${name}()`);
    const open = src.indexOf('{', start);
    let depth = 0;
    let i = open;
    for (; i < src.length; i += 1) {
        const c = src[i];
        if (c === '"') {
            i += 1;
            while (i < src.length && src[i] !== '"') {
                if (src[i] === '\\') i += 1;
                i += 1;
            }
        } else if (c === '{') depth += 1;
        else if (c === '}') {
            depth -= 1;
            if (depth === 0) return src.slice(open + 1, i);
        }
    }
    throw new Error(`unbalanced body ${name}`);
}

// Split a block into top-level statements / nested blocks.
export function* iterStatements(block) {
    let i = 0;
    let cur = '';
    while (i < block.length) {
        const c = block[i];
        if (c === '"') {
            const start = i;
            i += 1;
            while (i < block.length && block[i] !== '"') {
                if (block[i] === '\\') i += 1;
                i += 1;
            }
            cur += block.slice(start, i + 1);
            i += 1;
            continue;
        }
        if (c === '{') {
            // `{` opens a code block only in statement-leading position:
            // empty head (bare block) or right after for/if/else/while head.
            const head = cur.trim();
            const isBlock = head === '' || /(for|if|else|while|switch)\s*\([\s\S]*\)\s*$/.test(head) || /\belse\s*$/.test(head);
            let depth = 1;
            const start = i;
            i += 1;
            while (i < block.length && depth > 0) {
                if (block[i] === '"') {
                    i += 1;
                    while (i < block.length && block[i] !== '"') {
                        if (block[i] === '\\') i += 1;
                        i += 1;
                    }
                    i += 1; // step past the closing quote
                    continue;
                }
                if (block[i] === '{') depth += 1;
                if (block[i] === '}') depth -= 1;
                i += 1;
            }
            if (isBlock) {
                yield { type: 'block', head, body: block.slice(start + 1, i - 1) };
                cur = '';
            } else {
                // brace-initializer inside an expression: keep it inline
                cur += block.slice(start, i);
            }
            continue;
        }
        if (c === ';') {
            yield { type: 'stmt', text: cur + ';' };
            cur = '';
            i += 1;
            continue;
        }
        cur += c;
        i += 1;
    }
    if (cur.trim()) yield { type: 'stmt', text: cur };
}

