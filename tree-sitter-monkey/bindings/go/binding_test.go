package tree_sitter_monkey_test

import (
	"testing"

	tree_sitter "github.com/tree-sitter/go-tree-sitter"
	tree_sitter_monkey "github.com/0xcacti/interpreter/bindings/go"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_monkey.Language())
	if language == nil {
		t.Errorf("Error loading Monkey grammar")
	}
}
