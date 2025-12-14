# Lang Design

Want something easily suited to games programming.
Maybe first-class support for entities with a world position, sprite, and behaviours?

Can look up other entites by object easily.

e.g.

```c
entity Player {
    init {
        @speed = 3;
    }

    tick {
        new_pos = @position;
        if key("left") {
            new_pos.x -= @speed;
        } // etc

        foreach (w in entity Wall) {
            if w.contains(new_pos) {
                return;
            }
        }
    }
}
```

Could be fun to have first-class support for B&W graphics. Nice stylisation for game too:

```c
sprite {
    ####
    #  #
    #  #
    ####
}
```
