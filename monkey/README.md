## monkey 

Here is my go at implementing Monkey in rust.  Initially, I only implemented the interpreter 
as defined in his Interpreter Book. As I really enjoyed the book, 
I decided to give the Compiler Book a spin.  Now, the implementation includes 
both the original naively interpreted implementation, and the bytecode 
compiled + vm implementation.  As a treat, you can also run this project 
in the browser using wasm. 

### Usage

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
bytecode-interpreted (rather than sourcecode interpreted) and vm executed version of monkey, by supplying the --mode vm

``` 
monkey --mode vm
monkey --mode vm -i ./lib.monkey
monkey --mode vm ./main.monkey 

```

### Acknowledgements

Some of the code here was influenced by [ ThePrimeagen's ](https://github.com/ThePrimeagen) version which he abandoned 
two chapters into the book.  Much of the code is influenced by [monkey-wasm](https://github.com/shioyama18/monkey-wasm/tree/master).
Additionally, for the compiler implementation, I occasionally referenced [cymbal](https://github.com/shuhei/cymbal)

