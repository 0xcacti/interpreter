## MonkeyLang Interpreter in Rust

This repo contains an implementation of Thorsten Ball's Monkey Language in Rust. 
Initially, I only implemented the interpreter as defined in his Interpreter Book. 
As I really enjoyed the book, I decided to give the Compiler Book a spin.  Now, the 
implementation includes both the original naively interpreted implementation, and the 
bytecode compiled + vm implementation.  As a treat, you can also run this project 
in the browser using wasm. 

I used this project to learn rust, and I imagine it isn't the most idiomatic rust code.
Please feel free to rip into it.

### Instructions

I designed the interpreter binary to work much like the lua binary.  
You can run the interpreter (in naive-interactive mode) by simply using the binary name.

``` 
monkey
```

Additionally, you can supply a script to run or library to load before opening the repl 
with the -i flag. 

``` 
monkey -i ./lib.monkey
```

To simply execute a file, without using the repl, provide the path to the file as an argument. 

``` 
monkey ./main.monkey
```

Finally, you can run any of the above commands using the more performant, 
bytecode-compiled and vm executed version of monkey, by supplying the --mode vm

``` 
monkey --move vm
monkey --mode vm -i ./lib.monkey
monkey --mode vm ./main.monkey 

```

Some of the code here was influenced by [ ThePrimeagen's ](https://github.com/ThePrimeagen) version which he abandoned 
two chapters into the book.  Much of the code is influenced by [monkey-wasm](https://github.com/shioyama18/monkey-wasm/tree/master)



TODO: 
- Work on fixing the VM mutable borrow issue
- Figure out what you actually need to be implementing the tests to switch on - 
    - Right now you give both args as options, the example says to do something like 
    ```go 
    func testExpectedObject(
        t *testing.T,
        expected interface{},
        actual object.Object,
    ) {
        t.Helper()
        switch expected := expected.(type) {
        case int:
            err := testIntegerObject(int64(expected), actual)
            if err != nil {
                t.Errorf("testIntegerObject failed: %s", err)
            }
        }
    }
    ```

    - I simply use 
    ```rust 
        fn test_expected_object(expected: object::Object, actual: object::Object) {
            match expected {
                object::Object::Integer(expected) => validate_integer_object(actual, expected),
                _ => panic!("branch not covered"),
            }
        }
    ```
