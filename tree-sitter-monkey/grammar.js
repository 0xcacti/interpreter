/**
 * @file Tree-sitter parser for the Monkey programming language from Thorsten Bal
 * @author 0xcacti
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check
module.exports = grammar({
  name: "monkey",

  rules: {
    source_file: $ => repeat($._statement),

    _statement: $ => choice(
      $.let_statement,
      $.return_statement,
      $.expression_statement,
    ),


    let_statement: $ => seq(
      'let',
      field('name', $.identifier), 
      '=',
      field('value', $._expression),
      ';'
    ),

    return_statement: $ => seq(
      'return',
      field('value', $._expression),
      ';'
    ),

    expression_statement: $ => seq(
      $._expression,
      ';'
    ),

    _expression: $ => choice(
      $.identifier, 
      $.integer
    ),

    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,
    integer: $ => /[0-9]+/


  }
});
