import XCTest
import SwiftTreeSitter
import TreeSitterMonkey

final class TreeSitterMonkeyTests: XCTestCase {
    func testCanLoadGrammar() throws {
        let parser = Parser()
        let language = Language(language: tree_sitter_monkey())
        XCTAssertNoThrow(try parser.setLanguage(language),
                         "Error loading Monkey grammar")
    }
}
