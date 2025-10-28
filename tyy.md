# Mumbo Programming Language

this spec is a draft and a work-in-progress. there may be more things in the langauge then specified in the spec.

| Keyword     | Status   | Notes                       |
| :---------- | :------- | :-------------------------- |
| let         | existing | Bind name to value          |
| fn          | existing | Define function             |
| return      | existing | Exit function with value    |
| extern      | existing | Declare external symbol     |
| const       | existing | Read-only qualifier         |
| mut         | existing | writable qualifier          |
| anymut      | existing | Mutability generic          |
| compiletime | existing | Force compile-time storage  |
| runtime     | existing | Force runtime storage       |
| static      | existing | Persist in binary image     |
| type        | existing | First-class type value      |
| literal     | existing | Literal meta-type           |
| cast        | existing | Primitive conversion        |
| uninit      | existing | Uninitialized value literal |

| Keyword  | Status      | Notes               |
| :------- | :---------- | :------------------ |
| if       | prospective | Conditional branch  |
| else     | prospective | Alternate branch    |
| while    | prospective | Conditional loop    |
| loop     | prospective | Infinite loop       |
| break    | prospective | Exit loop           |
| continue | prospective | Next loop iteration |
| struct   | prospective | Product type        |
| enum     | prospective | Tagged union        |
| union    | prospective | Untagged union      |
| alias    | prospective | Type alias          |
| use      | prospective | Import symbols      |

| Keyword | Status | Notes              |
| :------ | :----- | :----------------- |
| module  | future | Module declaration |
| import  | future | Module dependency  |
| pub     | future | Public export      |
| export  | future | Re-export symbol   |

## grammar

See `grammar.md`.

## primitive integers

primitive integer types from rust directly carry over: `u8`, `i8`, `f32`, `u32`, `i32`.
isize and usize are parametric over their size and sized on monomorphization based on the target architecture pointer width.

## mutability

all types are by default trivially parametric over mutability unless specified with mutability fragment:
`{mutability?} {type}`
where mutability is optionally `mut` (writable, readable) or `const` (only readable) or to explicitize genericness `anymut`.
`mut` in Mumbo would be `mut` in rust and `var` in zig.
`const` in Mumbo would be a `let` without a `mut` in rust, and `const` in zig.
`u8` (or `anymut u8`) is a generic u8 that could stand in for `mut u8` or `const u8`.
if the final program after all monomorphization contains mutable values, all of those mutabilities will be assumed `const`. this means in Mumbo all `anymut` values are constant unless written to.

## pointer types and pointees

pointers exist, they are just `{mutability?} *{type}`. they are also parametric over mutability as you can see from the syntax.
pointer mutability describes the ability of a pointer's address to change. pointee mutability is contained within the pointee type.
when coming from other langauges, one may assume `*const u8` is a concrete type.
do not fall into this trap: if any type (including pointers) doesn't have a `const` or `mut` at the leftmost part, it is generic.
this means all of the below are valid pointer types:

concrete pointers:
const *const T
const *mut T
mut *const T
mut *mut T

pointers with 1 generic:
const *T (const *anymut T)
mut *T (mut *anymut T)
*const T (anymut *const T)
*mut T (anymut *mut T)

pointer with 2 generics:
*T (anymut *anymut T)

if the final program after all monomorphization contains mutable-generic pointers, all of those mutabilities will be assumed `const`. this means in Mumbo all mutable-generic pointees are constant unless written to.

writing through any generic pointer will inform type inference that the pointee type must be `mut`.
this is not the case with reading as every mutability permits reading.

## literals

literals are non-keyword data that is explicitly written in the source code. all of them can decay into values of specific types, one of them (uninit) must decay into a value or it doesn't make sense.
there is a separate literal type that can literally hold a literal for literal modification before coercion. (don't know why you would need that, but you can use it. i'd recommend using a calculator or a script to calculate the literal yourself)
values of type literal can't exist at runtime, nor can literals themselves. that mean's they're implicitly compiletime. in fact, all literal expressions have an implicit `compiletime` keyword before them, and you can explicitly write that if you want. it doesn't do anything.

## variance

variance from rust carries over. you can't access the `T` in `*const *mut T`,
easier grouping to see the pointee mutability: `anymut *(const *(mut T))`. (notice how this sentence itself is generic)

## tuples, arrays and zero sized types

tuples, arrays and ZSTs carry over from rust directly.

tuple types are `anymut (anymut ty1, anymut ty2, anymut ty3, ...)` and instantiations are `(val1, val2, val3)`.
`anymut ()` is a valid type and a valid value for the unit type.
array types are `anymut [N ty]` and instantiations are `[val1, val2, val3, val4]` or for automatic filling with a value `[N; val1]`.
the anymut denotes the mutability for each element in the array: you can't have arrays where the first item is mut and the second is const etc.
what you can have is this instead: `anymut [N (const T, mut T)]`. this will be `[const T, mut T, const T, mut T, ...]` in memory.

## types are first class citizens

the `type` type carries over form zig. types can exist at compiletime as values and their type is `type`.
their sole use is to write conditions for generic types for functions and ADT's. there is no `anytype` like in zig.

## pointer metadata

a 2-tuple containing first a pointer than anything is considered a wide pointer.
slicing operations return wide pointers where the metadata can be inferred (instead of the usual usize in rust).

```
let some_array: [u8 5] = runtime [10, 20, 30, 40, 50];
let another_array: [u8 4] = some_array[0..4];
let slice: (const *const u8, anymut usize) = &some_array[0..4];
// slice will become UB to access if you go beyond the backing array's (in the stack) length
slice.1 = 15;
// UB to read and write
slice.1 = 4;
// safe again

slice.0 += 15;
// pointer is invalid now
slice.0 -= 15;
// phew its valid now
```

## static

semantics of static values are mostly unspecified but the gist is that they live in the binary of the program. any static value
can decay into a non-static type, but non-static types can't become static types. non-static decay can happen behind pointers too.
static mutable values are stored in the data section, static immutable valeus in the rodata section etc.
static pointer values are no exception, they also live in the data or rodata.
the classic homemade C allocator usually uses a static `HEAD` variable. the type of that variable is this:
`(C) static Header *HEAD;` = `let HEAD: static mut *mut Header`: the pointer itself is supposed to be mutated each time you allocate, and the underlying buffer is not static as it is heap allocated.
an inverse would be:
`(Rust) let boxed: Box<&'static str>` = `let boxed: const *mut *static const u8`: the heap allocation (mutable pointer to heap) stores a pointer to rodata (immutable pointer to stack)
don't know why you would do that but you can also store a static pointer to a static:

```
let five_ref: const *static const usize = &5;
let six_ref: const *static const usize = &6;
let five_ref_controller: mut *static const *static const usize = &five_ref;
*five_ref_controller = &six_ref; // the entire world burns
```

## strings

there are many types of strings in Mumbo. string literals start with double quotes ("").
string literals are one of the 2 runtime-by-default literals. its type can change depending on the coercion.
string literals by themselves evaluate to arrays of bytes. if taken a reference, an the resulting array value will implicitly be static.
character literals can be written with single quotes ('') and state a byte value

```
let compiletime_str: compiletime anymut [13 u8] = compiletime "I live in compiletime!";
let bytearray: anymut [13 u8] = "I live in the stack!";
let c_string: const *static const u8 = &"I live in rodata.\0";
let c_auto_sentinel: const *static const u8 = &c"~ nulbyte at my end. ~ nulbyte at my end. ~ nul-nul-nul-nul nulbyte at my end!";
let mutable_c_string: const *static mut u8 = &"I live in data~nyan!\0";
let rust_style_slice_string: (const *static const u8, const usize) = &"I'm a pointer to a static byteslice in rodata lmao!";
let string: anymut *anymut u8 = &"I can coerce to any other pointed-string type by the way you use me!\0";
let pointer_to_array: const *anymut [_ u8] = &"I'm a pointer to a temporary in the stack!";

let letter_h: const literal = 'h';
let newline: const u8 = '\n';
```

## external functions

external functions can be declared by using the `extern` keyword.
you can also define external functions from Mumbo that will be exported.

```
extern fn printf(let format: const *static const u8, ...) -> ();

extern fn hello_mumbo() -> const *static const u8 {
    return &"Hello from Mumbo!";
}
```

## compile time execution

i lied, all types are also generic over whether they're in compile time or runtime.

```
// you can explicitly state a type's runtimeness: `u8` could be `compiletime u8` or `runtime u8`.
// the example below will give you a compile time mutable memory section.
let COMPILE_MEMORY: compiletime mut [1024 * 1024 u8] = [1024 * 1024; 0];`
// you can explicitly force a literal to coerce by prepending a `runtime` keyword before it.
let modifiable_literal: mut literal = "hello";
hello += "d";
let runtime_final: const [_ u8] = runtime modifiable_literal;
```

the example below may give you a compile time mutable memory section.
`let COMPILE_MEMORY: mut [1024 * 1024 u8] = [1024 * 1024; 0];`

`compiletime`ness and `static`ness are mutually exclusive. you either live in the compiler, or the binary.

`let MY_MAGIC: const u8 = compiletime 5;`
all compiletime primitive types (along with values of type `literal`) support evaluating their operations with other `compiletime` values.
`compiletime 5 + compiletime 10 == compiletime 15`

literals and values of type literal can be
that means you can have weird macros like this:

```
let dunno: compiletime mut literal = 0;

let zero = runtime dunno;
dunno += 1;
let one = runtime dunno;
```

```
let offset: comptime mut isize = compiletime -9;
... &array_end[offset] ...;
offset += 15;
... &array[offset] ...;
// but you cannot have this:
let final_offset = runtime offset; // WRONG
let final_offset = offset; // CORRECT
// because `runtime` makes *literals* go from compiletime into runtime, other values can already be implicitly converted to runtime values.
```

external functions never return `compiletime` values (unless they're defined and called in Mumbo at the same time) and can't take arguments that have `compiletime` in their type.
`compiletime` values will be demoted into runtime values when passed into external functions.

## uninitialized

zig's undefined carries over as `uninit`. `uninit` is one of the only 2 runtime-by-default literals. `uninit` doesn't have a type. it must coerce into a value of any other type.
`uninit` can also be coerced into a `compiletime` type. because of this, care must be taken to not have UB while compiling.
`let memory: compiletime (compiletime mut bool, compiletime mut u64) = (false, uninit); /* in reality this is this value: (compiletime false, compiletime uninit) */`
a shorter way to write the above:
`let memory: (mut bool, compiletime mut u64) = (false, uninit);`
boolean literals (and every other literal) are always compiletime so `mut bool`, uninit literals are always `runtime` so `compiletime mut u64`. compiletime tuples can be formed out of all-compiletime values.

## casts

primitive types can be casted with `cast`:

```
// a ['0'; 4096] manually made using literals
let buffer = [0; 4096];
let p_buffer: const *mut u8 = &buffer;

let addr: const usize = p_buffer cast usize;
```

## functions

functions are similar to zig and rust.

```
fn read(let buffer: const *mut u8) -> anymut usize {
    let ptr = buffer cast  const *const usize;
    *buffer
}
```
