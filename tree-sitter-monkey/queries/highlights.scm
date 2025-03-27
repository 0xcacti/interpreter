((let_statement "let") @keyword)
((return_statement "return") @keyword)
((if_expression "if") @keyword)
((if_expression "else") @keyword)

((function_expression "fn") @keyword)
((function_expression (identifier)) @variable.parameter) 
((function_expression body: (block)) @function)

((macro_expression "macro") @keyword)
((macro_expression (identifier)) @variable.parameter) 
((macro_expression body: (block)) @macro) 

((call_expression function: (identifier)) @function)

(identifier) @variable

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
