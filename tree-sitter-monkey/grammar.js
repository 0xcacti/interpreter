/**
 * @file T ree-sitter parser for the Monkey programming language from Thorsten Bal
 * @author 0xcacti
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "monkey",

  rules: {
    // TODO: add the actual grammar rules
    source_file: $ => "hello"
  }
});
