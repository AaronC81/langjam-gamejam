# Untitled Programming Language

This feature-incomplete, game-oriented, interpreted scripting language was created very hastily for [Langjam Gamejam](https://langjamgamejam.com/).

The concept is that _everything_ for your game - including graphics and audio - is defined in the language. No external files are needed.

# Basics

## Core

Statements end with semicolons.
Comments are `/* this kind of block comment */`.

For core data types, the language supports:

- Numbers: `42`, `3.14` - internally 64-bit floats
- Booleans: `true`, `false`
- Arrays: `[ 1, 2, 3 ]`
- Null: `null`

Some operations you can perform on this data:

- Core mathematical operations on numbers: `+`, `-`, `*`, `/`
- Comparisons on numbers: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Boolean operations on booleans (short-circuiting): `&&`, `||`

When the game starts, it executes the top-level `constructor`.
This is effectively your `main` function.
(Code cannot appear at the top-level of a file.)

Local variables can be (re-)assigned with `=`, and don't need any initial definition.

There is an `echo` expression to print objects to the console.

```
constructor {
    echo 42;

    result = 2 + 2;
    echo result;
}
```

## Sprites

Entities can draw graphics to the screen by using **sprites**.
(How entities can use sprites will be covered later.)

A sprite expression includes one of more whitespace-separated rows of pixels.
Each pixel is either `#` (black) or `.` (white).
All rows must be the same size.

For example, this sprite is a `+` symbol:

```
sprite {
    .#.
    ###
    .#.
}
```

Sprites have `.width()` and `.height()` functions to get their pixel dimensions as numbers.

## Sounds

Entities can play simple sine-wave audio tones by defining **sounds**.

A sound includes a duration in seconds, and a note to play (where A is concert-pitch).

For example, this will create the sound of an A note for 50 milliseconds:

```
sound { 0.05: A }
```

(Sharp/flat notes are not supported, only `A`-`G`.)

Creating the sound does not immediately play it.
Sounds have a `.play()` function to play the audio.

# Control Flow

Handle conditions using the `if` statement (no `else` though, sorry!)

```
if (health <= 0) {
    dead = true;
}
```

Loop over array items using the `each` statement:

```
arr = [1, 2, 3];
each x in (arr) {
    echo x;
}
```

# Entities

## Programming Model

All data is contained within **entities**.
After defining an entity, it can be **spawned** as many times as you like, with a `spawn` expression.
Each spawned copy has its own independent data.

An entity can include:

1. Logic which runs when the entity is spawned
2. Logic which runs periodically (`tick`/`draw`)
3. Variables which persist until the entity is destroyed
4. Functions which can be called by this or other entities

## Execution Lifecycle

Your top-level `constructor` should spawn any initial entities required for the game to work.
This is the only place where code is allowed _outside_ an entity.

After this, everything happens in **ticks**.
There are 30 ticks per second.

Every tick:

1. `tick` is executed for all entities, then
2. `draw` is executed for all entities

You can implement logic which changes over time (movement, animation, etc) by keeping track of state in variables between ticks.

## Defining Entities

Use an `entity` block to define an entity.

```
entity Something {
    /* Your entity goes here... */
}

/* Make sure it's spawned so it actually does something */
constructor {
    spawn Something;
}
```

To add logic, use a `constructor` block for the initial creation of the entity, and a `tick` block for every future tick:

```
entity Something {
    constructor {
        echo 42;
    }

    tick {
        echo 123;
    }
}
```

To store data across ticks, use `var` to define instance variables, which must begin with `@`.
Instance variables are initialised to `null` by default, so you may want to assign them a sensible default in the constructor.

```
entity Counter {
    var @count;

    constructor {
        @count = 0;
    }

    tick {
        @count = @count + 1;
    }
}
```

If you would like to organise logic within the entity, or expose logic to other entities, you can define functions with `func`:

```
entity ScoreTracker {
    var @score;

    constructor {
        @score = 0;
    }

    tick {
        this.add_score(1);
    }

    func add_score(amount) {
        @score = @score + amount;
    }
}

/* You can retain a reference to a spawned entity and call its functions */
constructor {
    score = spawn ScoreTracker;
    score.add_score(10);
}
```

Functions may also `return` values.

## Drawing With Entities

An entity can also draw one sprite to the screen.

To do this:

1. Define a `draw` block which returns a sprite
2. Define instance variables `@x` and `@y` for the position of the sprite

```
entity Plus {
    var @x, @y;

    constructor {
        @x = 10;
        @y = 10;
    }

    draw {
        return sprite {
            .#.
            ###
            .#.
        };
    }
}
```

> The encouraged model is that `tick` contains logic and `draw` just generates a sprite, but there's no firm restriction on this.
> `draw` can _technically_ do whatever you want.

## Deduplicating Logic Between Entities

You might end up with certain definitions which would be useful in many different entities.

Within an entity definition, the `use` keyword copies all definitions from another entity.
So, you can create a template entity, and then `use` it in other entities.

```
entity Template {
    var @a;

    constructor {
        @a = 0;
    }

    func something_useful() {
        /* ... */
    }
}

entity Something {
    use Template;
}
```

Specifically, `use` will:

* Copy all variable and function definitions
* Merge `constructor` and `tick` definitions
* Import the `draw` definition
    * (`draw` definitions cannot be merged; it will error if more than one is defined)

# Standard Library

## Entities

If you have an `entity X`, then `X.all()` returns an array of all currently-existing instances of `X`.

## Input

The following functions exist to check whether certain keys are being held:

* `Input.up_pressed()`
* `Input.down_pressed()`
* `Input.left_pressed()`
* `Input.right_pressed()`
* `Input.z_pressed()`
* `Input.x_pressed()`

## Display

`Display.width()` and `Display.height()` get the pixel dimensions of the game display.

## Mathematics

`Math.random_int(start, end)` will return a random integer between `start` and `end`, inclusive on both sides.

# Shortcomings

This language was pretty much implemented as I needed stuff, so if I didn't need it, it's not here:

* Features:
    * No strings
    * Lacking control flow - no support for `else`, range-based loops, or `while` loops
    * No line comments
    * Constructor parameters are not supported
    * Small standard library
* Design:
    * No formal import mechanism - all files get loaded in alphabetical order of their filename
    * `tick`/`draw` distinction is somewhat arbitrary
        * Some engines use this to `tick` fast and `draw` slow, but we don't
    * `use` is more confusing than inheritance
* Implementation:
    * Very bad parse errors
    * Interpreter is abhorrently slow
