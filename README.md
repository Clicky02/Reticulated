# Reticulated

Reticulated is a statically typed, compiled programming language with a syntax similar to Python. It is designed to be multi-platform by leveraging LLVM for code generation. This project is being developed as a part of my Master's CAPSTONE project.

## Features

-   **Statically Typed**: All variables and expressions have a type that is known at compile time.
-   **Python-like Syntax**: The language syntax is inspired by Python, making it easy to read and write.
-   **Multi-Platform**: By using LLVM, the compiler can generate code for multiple platforms.

## Example Code

```ret
name: string = "Gareth"
age: int = 23

if age < 0 or age > 200 {
    print("Invalid age")
} else if age < 18 {
    print("You are a minor")
} else {
    print("You are an adult")
}

fn hello(name: string) -> string {
    return "Hello, " + name + "!"
}
```

## WIP Grammar

```plaintext
program -> statement*
statement -> (declaration | assignment | function_declaration | extern_function | if_statement | return_statement | expression | struct_declaration | while_loop) "\n"
declaration -> IDENTIFIER ":" IDENTIFIER "=" expression
assignment -> IDENTIFIER ("." IDENTIFIER)*  "=" expression
function_declaration -> "def" IDENTIFIER "(" parameters ")" "->" IDENTIFIER block
extern_function -> "extern" "def" IDENTIFIER "(" parameters ")" "->" IDENTIFIER
if_statement -> "if" expression block ("else" "if" expression block)* ("else" block )?
return_statement -> "return" expression
block -> "{" statement* "}"
parameters -> ("self" ",")? (parameter ("," "*"? parameter)*)?
parameter -> IDENTIFIER ":" IDENTIFIER
struct_declaration -> "struct" IDENTIFIER "{" struct_field* "}"
struct_field -> IDENTIFIER: IDENTIFIER "," | function_declaration
while_loop -> "while" expression block

expression -> equality
logical -> equality ( ("or" | "and") equality )*
equality -> comparison ( ("!=" | "==") comparison )*
comparison ->  term ( ( ">" | ">=" | "<" | "<=" ) term )*
term -> factor ( ( "-" | "+" ) factor )*
factor -> unary ( ( "/" | "*" | "%" ) unary )*
unary -> ( "!" | "-" ) unary | invoke
invoke -> (invoke | access)  "(" arguments ")"
access -> (access | primary) "." IDENTIFIER
primary -> IDENTIFIER | INTEGER | FLOAT | STRING | BOOL | NONE | "(" expression ")"

arguments -> (expression ("," expression)*)?
```

## Project Goals

In no specific order:

### MVP

-   [x] Define initial grammar
-   [x] Create example code
-   [x] Implement lexer
-   [x] Implement parser
-   [x] Integrate with LLVM for code generation

### Additional Functionality

-   [ ] More language features
    -   [ ] Comments
    -   [x] While loop
    -   [ ] For Loop
    -   [ ] First-class functions
    -   [x] Structs
    -   [ ] Classes/Inheritance/Polymorphism
    -   [ ] Imports
    -   [ ] List Comprehensions
    -   [ ] Dictionay Comprehensions
    -   [ ] Lambda functions
    -   [ ] Contexts
-   [x] Implement type checker
-   [ ] Basic Standard Library Features
-   [ ] Write documentation
-   [ ] Create test suite

### Project Cleanup

-   [ ] Better error reporting and handling.
-   [ ] Add more comprehensive command line argument support
