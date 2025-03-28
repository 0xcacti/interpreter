((let_statement "let") @keyword)
((return_statement "return") @keyword)
((if_expression "if") @keyword)
((if_expression "else") @keyword)
((function_expression "fn") @keyword)
((macro_expression "macro") @keyword)

((let_statement name: (identifier) @variable.global)
  (#not-has-ancestor? @variable.global block function_expression macro_expression))
((let_statement name: (identifier) @variable.local)
 (#has-ancestor? @variable.local block function_expression macro_expression))

((identifier) @variable)

((function_expression (identifier)) @variable.parameter) 
((function_expression body: (block)) @function)

((macro_expression (identifier)) @variable.parameter) 
((macro_expression body: (block)) @function) 

(quote_unquote) @function.builtin
(builtin) @function.builtin  
((call_expression function: (identifier)) @function.call)
((call_expression function: (builtin)) @function.builtin)
((call_expression function: (quote_unquote)) @function.builtin)


(prefix_expression
  operator: (_) @operator)

(infix_expression
  operator: (_) @operator)

(integer) @number
(boolean) @boolean
(string) @string
((array "[" @punctuation.bracket)
  ("]" @punctuation.bracket))
(array) @type

((hash "{" @punctuation.bracket)
  ("}" @punctuation.bracket))
(hash) @type

(null) @constant

["(" ")" "{" "}" "[" "]" "," ";"] @punctuation.delimiter
[":" "="] @punctuation.special

(comment) @comment
