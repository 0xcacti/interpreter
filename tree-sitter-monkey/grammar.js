/**
 * @file Tree-sitter parser for the Monkey programming language from Thorsten Bal
 * @author 0xcacti
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check
module.exports = grammar({
  name: "monkey",

  extras: $ => [$.comment, /\s/],
  word: $ => $.identifier,

  rules: {
    source_file: $ => repeat($._statement),

    _statement: $ => seq(
      choice(
        $.let_statement,
        $.return_statement,
        $.expression_statement,
      ),
      optional(';')
    ),

    let_statement: $ => seq(
      'let',
      field('name', $.identifier), 
      '=',
      field('value', $._expression),
    ),

    return_statement: $ => seq(
      'return',
      field('value', $._expression),
    ),

    expression_statement: $ => seq(
      $._expression,
    ),

    _expression: $ => choice(
      $.identifier, 
      $.literal,
      $.prefix_expression,
      $.infix_expression,
      $.if_expression,
      $.function_expression, 
      $.macro_expression,
      $.call_expression,
      $.index_expression,
    ),

    literal: $ => choice(
      $.integer,
      $.boolean,
      $.string,
      $.array,
      $.hash,
      $.null,
    ),

    identifier: () => /[a-zA-Z_][a-zA-Z0-9_]*/,
    integer: () => /[0-9]+/,
    boolean: () => choice('true', 'false'),
    string: () => /"([^"\\]|\\.)*"/,
    null: () => 'null',
    array: $ => seq('[', repeat($._expression), ']'),
    hash: $ => seq('{', repeat(seq($._expression, ':', $._expression)), '}'),

    prefix_operator: () => choice('!', '-'),
    prefix_expression: $ => prec.right(5, seq(
      field('operator', $.prefix_operator),
      field('right', $._expression),
    )),

    infix_expression: $ => choice(
      prec.left(1, seq(field('left', $._expression), field('operator', choice('==', '!=')), field('right', $._expression))),
      prec.left(2, seq(field('left', $._expression), field('operator', choice('>', '<')), field('right', $._expression))),
      prec.left(3, seq(field('left', $._expression), field('operator', choice('+', '-')), field('right', $._expression))),
      prec.left(4, seq(field('left', $._expression), field('operator', choice('*', '/')), field('right', $._expression))),
    ),

    if_expression: $ => seq(
      'if', 
      '(',
      field('condition', $._expression),
      ')',
      '{',
      field('consequence', $.block),
      '}',
      optional(seq(
        'else',
        '{',
        field('alternative', $.block),
        '}'
      ))
    ),

    function_expression: $ => prec(6, seq(
      'fn',
      '(', 
      optional(seq(
          $.identifier, 
          repeat(seq(',', $.identifier)), 
          optional(','), 
        )),
      ')',
      field('body', $.block),
    )),

    macro_expression: $ => prec(6, seq(
      'macro',
      '(',
      optional(seq(
          $.identifier, 
          repeat(seq(',', $.identifier)), 
          optional(','), 
        )),
      ')',
      field('body', $.block),
    )),

    call_expression: $ => prec(7, seq(
      field('function', $.identifier), 
      '(',
        optional(seq(
          $._expression, 
          repeat(seq(',', $._expression)),
          optional(','),
        )),
      ')',
    )),

    index_expression: $ => prec(8, seq(
      field('indexable', $._expression),
      '[',
      field('index', $._expression),
      ']',
    )),

    block: $ => seq('{', repeat($._statement), '}'),

    comment: () => token(
      choice(
        seq('//', '/.*/'),
        seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, "/"),
      )
    ),
  }
});
