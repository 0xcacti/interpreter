((let_statement "let") @keyword)
((return_statement "return") @keyword)
((if_expression "if") @keyword)
((if_expression "else" @keyword)? @keyword)

(identifier) @variable

(prefix_expression
  operator: (_) @operator)

(infix_expression
  operator: (_) @operator)

(integer) @number
(boolean) @boolean
(string) @string

["(" ")" "{" "}" "[" "]" "," ";"] @punctuation.delimiter
[":" "="] @punctuation.special

(comment) @comment
