// swift-tools-version:5.3
import PackageDescription

let package = Package(
    name: "TreeSitterMonkey",
    products: [
        .library(name: "TreeSitterMonkey", targets: ["TreeSitterMonkey"]),
    ],
    dependencies: [
        .package(url: "https://github.com/ChimeHQ/SwiftTreeSitter", from: "0.8.0"),
    ],
    targets: [
        .target(
            name: "TreeSitterMonkey",
            dependencies: [],
            path: ".",
            sources: [
                "src/parser.c",
                // NOTE: if your language has an external scanner, add it here.
            ],
            resources: [
                .copy("queries")
            ],
            publicHeadersPath: "bindings/swift",
            cSettings: [.headerSearchPath("src")]
        ),
        .testTarget(
            name: "TreeSitterMonkeyTests",
            dependencies: [
                "SwiftTreeSitter",
                "TreeSitterMonkey",
            ],
            path: "bindings/swift/TreeSitterMonkeyTests"
        )
    ],
    cLanguageStandard: .c11
)
