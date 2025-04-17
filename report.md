## Abstract

<!--
# Creating a Simple Multi-Platform Programming Language

## Instructions

Sections
- abstract that succinctly describes the problem addressed, the methods used, and the results
- introduction that provides sufficient background to allow the reader to understand the problem, the constraints and relevant characteristics of the project
- methods (approach or analysis, as appropriate) that describe how the problem was addressed; this section should provide some details on how the skills and knowledge gained through the MEng program contributed to the solution
- results obtained through the project,
- discussion of the efficacy of the approach, lessons learned through the project, areas for improvement, additional work that could be performed
- bibliography of references cited.

Capstone reports should not exceed 10 pages, double-spaced, 11pt font, and one-inch margins.
Appendices with code or graphs, for example, can be included and cited in the body of the report.
-->

This capstone project presents the design and implementation of a prototype compiler for a new programming language called Reticulated. Reticulated is a statically-typed, compiled language that closely mirrors Python's syntax. By maintaining Python’s familiar and accessible syntax, Reticulated aims to be user-friendly while also providing the benefits of static typing and compilation.

The compiler is implemented entirely in Rust and leverages LLVM to generate efficient machine code targeting multiple platforms, including macOS, Linux, and Windows. Core components of the compiler include a custom-built lexer, parser, and code generator. The language currently supports a wide range of fundamental constructs (such as functions, data structures, control flow, and operator overloading), and a handful of basic types (integer, float, string, and boolean). Memory management is handled automatically through reference counting, eliminating the need for a garbage collector or manual memory management.

This project has two primary goals: (1) to create a proof-of-concept for an accessible, cross-platform, compiled programming language, and (2) to gain hands-on experience with compiler design and implementation. The result is a functional prototype that demonstrates the feasibility and potential of Reticulated as a performant, user-friendly language and provides a solid foundation for potential future development.

## Introduction

As its medium, programming languages are the foundation of software development. Unsurprisingly, beginners in the field often use simple, abstract languages. Among these, Python is arguably the most popular language [1]. It is powerful, simple, and easy to learn. However, Python is not ideal for all use cases. In particular, Python is dynamically typed and interpreted, which can lower performance and make some bugs more difficult to catch.

This project centers on the development of a cross-platform compiler for a subset of the Reticulated language. This language aims to be similar to Python in syntax and semantics while also providing the benefits of static typing and compilation. Specifically, the project had two main objectives: (1) to build a cross-platform, compiled language that is accessible and easy to use, and (2) to explore and apply key principles of compiler design and implementation.

Although the compiler does not support every language feature, it successfully implements fundamental constructs such as assignments, functions, and if statements, along with basic data types. Additionally, the prototype supports cross-platform compilation by leveraging LLVM in its backend. LLVM—a comprehensive collection of compiler toolchain technologies [2]—is used to generate efficient machine code for multiple platforms and architectures. Overall, the compiler serves as a proof-of-concept for the Reticulated language and can serve as a foundation for potential future work.

### Overview of Reticulated

Reticulated is a statically typed, compiled programming language designed to be simple and accessible. The language supports a range of features common in general-purpose programming languages, including:

-   Primitive types: integers, floats, booleans, strings, and None (a null-like value).
-   Control flow constructs: if/else, while loops.
-   Functions with optional type-annotated parameters and return types.
-   Structs that allow user-defined data types and encapsulate both data and behavior.

Reticulated's syntax closely mirrors Python, using nearly identical keywords, operators, and constructs [3]. However, there are a few notable exceptions:

-   Braces are used to denote blocks instead of indentation.
-   Functions can be overloaded. This allows multiple functions with the same name to coexist, as long as their parameter lists differ in type or number of parameters.
-   Type hints are mandatory. This is an optional feature in Python, but necessary for Reticulated as it is statically typed.

Furthermore, operations in Reticulated are implemented as dunder method calls, similar to Python [4] (e.g., `__add__`, `__sub__`). For example, the expression a + b is internally translated into a function call `a.__add__(b)`. During code generation, these operations are resolved by looking up the appropriate dunder method based on the left-hand operand’s type. If the method exists, the compiler emits a call to the corresponding function. This system allows not only built-in types but also user-defined types to customize behavior for arithmetic, comparison, and string operations. A simple Reticulated example program demonstrating these features can be found in [Appendix A](#appendix-a---example-program).

## Methods

### Grammar

Before writing the compiler, I first had to create a grammar that defined the syntax of the language. This grammar outlines the rules for how valid Reticulated programs are structured, and it is a key component to creating an effective parser. Furthermore, I ensured the grammar was context-free, unambiguous, and LL(1). This allows Reticulated programs to be parsed left-to-right, unambiguously, and with a single lookahead token.

<!-- For more information on the grammar, see [Appendix B](#appendix-b---compilers-grammar). -->

<!-- ### Compiler

The Reticulated compiler is written in Rust and divided into 3 major components: the lexer, the parser, and the code generator. The lexer is responsible for taking in source code and converting it into a series of tokens. The parser is responsible for taking the outputted tokens and converting it into an abstract syntax tree (AST). Finally, the code generator takes the AST and converts it into LLVM IR, which LLVM can convert into machine code for the target platform. The following sections will describe each component in more detail. -->

### Lexer

The lexer is the first of three components in the compiler. It takes in source code and chunks it into discrete parts called tokens, which are easier for the parser to work with. Tokens are the smallest unit of meaning in a programming language. They represent keywords, identifiers, literals, operators, and any other symbol that can appear in source code. Whitespace and comments are ignored in most cases. Figure 1 and Figure 2 show an example of source code and its token representation respectively.

```py
name: str = input("What is your name")
```

<figcaption>An example code snippet in the Reticulated language.</figcaption>

```plaintext
Identifier("name") [1:1, 1:5]
Colon [1:5, 1:6]
Identifier("str") [1:7, 1:10]
Operator(Assign) [1:11, 1:12]
Identifier("input") [1:13, 1:18]
OpenParenthesis [1:18, 1:19]
Literal("What is your name") [1:19, 1:38]
CloseParenthesis [1:38, 1:39]
EOF [1:39, 1:40]
```

<figcaption>The token representation of the code snippet from Figure 1.</figcaption>

The lexing process is performant and almost entirely context-independent. It works by reading characters from the input one at a time until the pattern for a token is found. Once a pattern is matched, the lexer invokes a handler to consume the characters from the input and construct an instance of the corresponding token object. If an unknown pattern or invalid sequence is encountered, the lexer raises a syntax error. Since the lexer is context-independent, it only passes over the input once, meaning its time complexity is linear relative to the length of the source code. This approach limits the types of tokens that can be created, but it is sufficient for Reticulated's needs.

### Parser

The parser is the second component of the compiler. It takes in tokens from the lexer and converts them into an abstract syntax tree (AST). The AST is a tree representation of the source code, where each node represents a construct in the language. Figure 3 shows an example AST produced by the parser.

```
Declaration {
    identifier: "name",
    type_identifier: "str",
    expression: Invoke(
        Primary(Identifier("input")),
        [ Primary(String("What is your name")) ]
    )
}
```

<figcaption>The AST produced by the parser for the source code in Figure 1.</figcaption>

A Reticulated program consists primarily of a series of statements. The parser implements statements as a Rust enum, where each variant represents a different type of statement. These types include variable declarations, variable assignments, function definitions, if statements, return statements, expression statements, struct declarations, and while loops. The children of these statements are their components. For example, a while loop has two children: a condition and a body. Similar to statements, expressions are also implemented as a Rust enum. Each expression type has an enum variant, and the children of these expressions are their components. For example, a binary operation has three children: a left side expression, an operator, and a right side expression. The expression enum also contains variants for literals, identifiers, and function invocations.

The parser utilizes recursive descent parsing to build the AST. This is an efficient and relatively simple parsing strategy [5]. However, it requires an unambiguous, context-free grammar with no left recursion. A recursive-descent parser is implemented by creating a function for each non-terminal in the grammar. Each function is responsible for parsing a specific type of statement or expression, recursively using the other functions when necessary. Parsing begins at the top-level production rule, which parses the entire program.

### Code Generation

Code generation is the third and most complex component of the compiler. Generally, code generation is the process of taking an AST and converting it into machine code. However, the Reticulated compiler generates LLVM IR instead of directly generating machine code. It does this by utilizing the Inkwell library, a safe Rust wrapper around the LLVM C++ API [6]. LLVM IR is a low-level programming language similar to assembly [7]. However, it is platform-independent and has additional rules, restrictions, and capabilities. By generating LLVM IR, the Reticulated compiler can target multiple platforms and architectures.

#### Environment

The environment is an essential data structure to the code generation process. Its purpose is to keep track of the environment surrounding the currently compiled code. This includes the current function, available functions, available types, and all active scopes. The environment is used to resolve identifiers, check types, and manage scopes.

#### Built-ins

The first step in the code generation process is to create the built-in types and functions. As previously mentioned, the Reticulated prototype compiler supports 5 basic data types: integers, floats, strings, booleans, and None. For each basic data type, the compiler creates a corresponding LLVM type and adds the type information to the environment. Furthermore, the compiler also creates several built-in functions. Some of these functions are type-associated functions (e.g. `int.__add__(int)`, `float.__sub__(float)`, etc.), while others are standalone functions (e.g. `print`, `input`, etc.). Each is manually built in LLVM IR using the Inkwell library and then added to the environment.

#### General Strategy

The code generation process works by recursively compiling each block. A block can be a function body, an if statement, a while loop, or any other construct that contains statements. The first block is the scripting code (made of every top-level statement). Each block is compiled recursively in two steps: (1) the preprocessing step and (2) the compilation step. The preprocessing step is responsible for adding every function and struct to the environment and creating their LLVM declarations. The compilation step is responsible for actually generating the LLVM IR for the block's logic. This two-step approach is necessary for supporting features like recursive functions and forward references.

During the compilation step, each statement is translated into LLVM IR by matching over the statement type and generating the corresponding IR instructions. This typically involves compiling child expressions and using their results in the instructions. For example, a variable declaration statement would compile its right-hand expression, create a variable in the local scope, and then store the expression's result in the created variable.

Expressions are compiled recursively and follow the structure defined by the AST. In the code generator, the result of compiling an expression is always a pointer to the resulting value and the resulting value's type ID. This system is a natural fit for Reticulated's memory management model, and it also allows for intuitive compiler design because all types of expressions/values are handled in the same way.

#### Reference Counting

Every object or value in Reticulated is reference counted. This means that every object is allocated on the heap with an associated reference count. When a new reference to the object is created, the reference is incremented, and when a reference is destroyed, the reference count is decremented. When the reference count reaches zero, the object is deallocated. This system allows Reticulated to have automatic memory management without the need for a garbage collector.

For each type, a reference count field is automatically added to the end of its struct. Additionally, user-inaccessible `copyptr` and `freeptr` functions are created for each type on compilation. These functions are then called anytime a reference is created or destroyed respectively. This strategy allows for automatic memory management and simplifies the code generation process because it allows the compiler to treat all types uniformly. However, it does add some overhead to the generated code both in terms of execution time and binary size.

### Executable Creation

The final step in the compilation process is to create the executable. First, this step takes the LLVM IR from the code generator and writes it to a file (as a compilation artifact). Then, it runs the LLVM optimizer (`opt`) on the IR to create optimized bytecode IR. The optimized IR is then passed to `llc` (the LLVM compiler), which creates an object file for the target platform. Finally, `clang` links the object file to create an executable.

### Knowledge and Skills Used

Although I did not take any compiler courses throughout my time in the CS MEng program, the classes I took and the skills I developed were still helpful and applicable to this project. For one, my Operating Systems coursework helped me understand how computers, C code, and assembly work at a low level, all of which were directly applicable to this project. Additionally, my coursework in Software Testing and Requirements Engineering helped me understand how to test the compiler and manage the project. Lastly, my coursework in Advanced Algorithms helped me design the parser and code generator efficiently.

## Results

The prototype compiler successfully parses and compiles a variety of Reticulated programs. Specifically, it supports the following language constructs:

-   Variable declarations and assignments.
-   Function definitions and calls.
-   Struct definitions and method calls.
-   If statements and else statements.
-   While loops.
-   Most binary and unary operators, including arithmetic, comparison, and logical operators.

The compiler also successfully implements a number of builtin types and functions, including:

-   The `print`, `input`, and `ref_count` functions.
-   The `int`, `float`, `str`, `bool`, and `None` types.
-   The `__add__` (`+`), `__sub__` (`-`), `__mul__` (`*`), `__div__` (`/`), `__pow__` (`**`), `__mod__` (`%`), `__gt__` (`>`), `__ge__` (`>=`), `__lt__` (`<`), and `__le__` (`<=`) dunder methods for the float and int types.
-   Type casting dunder methods for primitive types (e.g. `__str__`, `__int__`, etc.).

Furthermore, the compiler has cross-platform support by leveraging LLVM in its backend. The compiler has been tested on macOS, Linux, and Windows, and it works correctly on all three platforms. It also generates relatively efficient LLVM IR in a way that is easily extendable for future development. Lastly, the compiler also has automatic memory management for all types through the use of reference counting. The code for all parts of my compiler can be found [here](https://github.com/Clicky02/Reticulated).

## Discussion

### Efficacy of Approach

The approach taken in this project was effective in creating a simple, multi-platform programming language. LLVM in particular uniquely allowed for efficient code generation and cross-platform support. Additionally, using Rust as the implementation language may not have been optimal in terms of development time, but it did provide safety and performance benefits. The design of the lexer, parser, and code generator was also effective in creating a simple and extensible compiler. I do not expect a rewrite to be necessary to add more complex features like polymorphism, first-class functions, and closures.

### Future Work

Many features could be added in future work. The most immediate need is for lists and dictionaries. These are common data structures in most programming languages, and they would be useful in Reticulated. This work involves adding support for generic types, related dunder methods (e.g. `__getitem__`, `__setitem__`, `__len__`, etc.), and list/dictionary literals. Additionally, the compiler could be extended to support more complex features like imports, classes/polymorphism, first-class functions, closures, variable argument functions, comprehensions, exceptions, external function support, and more. Additionally, the language needs a larger standard library for common tasks like file I/O, networking, and data manipulation. These changes would make the language more powerful, flexible, and useful.

### Lessons Learned

Through this project, I learned a lot about software engineering and compilers. For one, I gained experience in designing and implementing a compiler from scratch. This includes insight into how compilers work in general, a field with which I had limited prior experience. Furthermore, working on the compiler deepened my understanding of Rust and enhanced my overall software engineering skills. Throughout the project, I had to make many design decisions, which helped me learn about design patterns and best practices in Rust and software development in general. Additionally, interacting with low-level computer concepts through LLVM enhanced my knowledge of system-level internals.

## Bibliography

<pre>
[1]  P. Carbonnelle, “PYPL PopularitY of Programming Language index,” Github.io, 2023. https://pypl.github.io/PYPL.html

[2]  LLVM Foundation, “The LLVM Compiler Infrastructure Project,” Llvm.org, 2019. https://llvm.org/

[3]  Python Software Foundation, “The Python Tutorial — Python documentation,” Python.org, 2025. https://docs.python.org/3/tutorial/index.html

[4]  Python Software Foundation, “3. Data model,” Python documentation, 2025. https://docs.python.org/3/reference/datamodel.html#special-method-names

[5]  R. Nystrom, Crafting interpreters. United States? Genever Benning, 2021.

[6]  TheDan64, “GitHub - TheDan64/inkwell: It’s a New Kind of Wrapper for Exposing LLVM (Safely),” GitHub, Aug. 04, 2024. https://github.com/TheDan64/inkwell (accessed Apr. 11, 2025).

[7]  LLVM Foundation, “LLVM Language Reference Manual — LLVM 16.0.0git documentation,” llvm.org. https://llvm.org/docs/LangRef.html
</pre>

<div style="page-break-after: always"></div>

## Appendix

### Appendix A - Example Program

```py
struct Vec {
    x: float,
    y: float,

    def __add__(self, other: Vec) -> Vec {
        return Vec(self.x + other.x, self.y + other.y)
    }

    def __add__(self, other: float) -> Vec {
        return Vec(self.x + other, self.y + other)
    }

    def __str__(self) -> str {
        return  "Vec(" + str(self.x) + ", " + str(self.y) + ")"
    }

    def dot(self, other: Vec) -> float {
        return self.x * other.x + self.y * other.y
    }
}

a: Vec = Vec(0.0, 0.0)
b: Vec = Vec(0.0, 0.0)
while True {
    a.x = float(input("What is the x value for Vector A? "))
    a.y = float(input("What is the y value for Vector A? "))
    b.x = float(input("What is the x value for Vector B? "))
    b.y = float(input("What is the y value for Vector B? "))

    c: Vec = a + b
    dot: float = a.dot(b)

    print("A + B = " + str(c))
    print("The dot product of A and B is " + str(dot) + ".")
}
```

This is an example program for adding and taking the dot product between two vectors.

<!--
### Appendix B - Compiler's Grammar

```plaintext
program -> statement*
statement -> (declaration | assignment | function_declaration | if_statement | return_statement | expression | struct_declaration | while_loop) "\n"
declaration -> IDENTIFIER ":" IDENTIFIER "=" expression
assignment -> IDENTIFIER ("." IDENTIFIER)*  "=" expression
function_declaration -> "def" IDENTIFIER "(" parameters ")" "->" IDENTIFIER block
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
arguments -> (expression ("," expression)*)?
access -> (access | primary) "." IDENTIFIER
primary -> IDENTIFIER | INTEGER | FLOAT | STRING | BOOL | NONE | "(" expression ")"
```

This is the prototype compiler's grammar for parsing Reticulated programs. The first block contains the production rules for the different types of statements supported by the language. Then, the second block defines the expression grammar, which handles built-in operations and operator precedence. It uses a hierarchy of production rules to parse each expression and ensure that mathematical expressions will follow the standard order of operations (with some additional operations like accessing a struct or invoking a function). Furthermore, the expression grammar also contains literals for the five supported basic data types: integers, floats, strings, booleans, and None (a null-like value). -->

<style>
.markdown-body  {
    font-family: serif;
    line-height: 2;
    counter-reset: h2counter;

}

.markdown-body h2, .markdown-body h3, .markdown-body h4, .markdown-body h5, .markdown-body h6 {
  margin-bottom: 8px;
  padding-bottom: 0;
}

.markdown-body p, .markdown-body ul  {
    font-size: 11pt;
    margin-bottom: 8px;
}

.markdown-body ul  {
    line-height: 1.8;
}



.markdown-body h1 {
    font-size: 20pt;
    border-bottom: 0px;
}

#abstract {
        counter-reset: figcounter;

}

.markdown-body h2 {
    font-size: 16pt;
    border-bottom: 0px;
    counter-reset: h3counter;
}

.markdown-body h2:before {
    counter-increment: h2counter;
    content: counter(h2counter)" ";
}

.markdown-body h3 {
    font-size: 14pt;
    counter-reset: h4counter;
}

.markdown-body h3:before {
    counter-increment: h3counter;
    content: counter(h2counter)"."counter(h3counter)" ";
}

.markdown-body h4 {
    font-size: 12pt;
}

.markdown-body h4:before {
    counter-increment: h4counter;
    content: counter(h2counter)"."counter(h3counter)"."counter(h4counter)" ";
}

.markdown-body pre {
    break-inside: auto;
    font-size: 10pt;
    padding: 8px 16px;
}

.markdown-body pre code {
    font-size: 10pt;
}

.markdown-body li + li {
  margin-top: 0;
}

figcaption {
    margin-top: -12px;
    margin-bottom: 12px;
    font-style: italic;
}

figcaption:before {
    font-weight: bold;
    counter-increment: figcounter;
    content: "Figure "counter(figcounter)": "
}
</style>
