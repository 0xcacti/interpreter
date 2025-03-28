// Single-line comment

/* Multi-line
   comment */

// Let statements with various literals
let x = 42;             // Integer literal
let y = true;           // Boolean literal
let z = "hello";        // String literal
let n = null;           // Null literal (once fixed)
let arr = [1, 2, 3];    // Array literal
let obj = {"a": 1, "b": "two"};  // Hash literal

// Function definition
let add = fn(a, b) {
    return a + b;       // Return statement with infix expression
};

// Macro definition
let double = macro(val) {
    return val * 2;     // Macro body with infix expression
};

// Prefix and infix expressions
let neg = !true;        // Prefix expression
let sum = 5 + 3;        // Infix addition
let eq = x == 10;       // Infix equality
let gt = y > false;     // Infix comparison

// If expression
let result = if (x > 0) {
    "positive"          // String literal as consequence
} else {
    "non-positive"      // String literal as alternative
};

// Function call
let total = add(4, 5);  // Call expression


// Index expression
let first_idx = arr[0];     // Array indexing

// Nested expressions
let complex = if (total != null) {
    echoln(first_idx);
    first(arr);
    let temp = {"key": 5 + sum};  // Nested hash and infix
    temp["key"];                     // Index expression in block
} else {
    0
};

