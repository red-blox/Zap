<script setup lang="ts">
const example = `funct Test = {
    call: Async,
    args: struct {
        foo: u8,
        bar: string
    },
    rets: enum { Success, Fail }
}`

const multiArgsRetsExample = `funct MultipleArgsRets = {
    call: Async,
    args: (boolean, u8),
    rets: (boolean, string)
}`
</script>

# Functions

Functions are another method of communication where the client can send arguments and have them returned by the server. For security, Zap only supports Client -> Server -> Client functions, not Server -> Client -> Server.

## Defining Functions

Functions are defined in your config file using the `funct` keyword.

<CodeBlock :code="example" />

As you can see they have three fields. Let's go over them one by one:

### `call`

This field determines how the function is listened to on the server. The function will take the `args` as parameters and return `rets`.

- `Async` functions can be listened to by one function, and they are called asynchronously.
- `Sync` functions can be listened to by one function, and they are called synchronously.

::: danger
Synchronous functions are not recommended, and should only be used when performance is critical.

- If a synchronous function callback yields it will cause **undefined and game-breaking behavior**.
- If a synchronous function callback errors it will cause **the packet to be dropped**.

Use synchronous functions with extreme caution.
:::

### `args`

This field determines the data that is sent to the server. It can be any [Zap type](./types.md).

- If the client doesn't send any data, the `args` field should be excluded.
- You can pass multiple arguments to the function by separating each type with a comma and wrapping them all in parentheses:

<CodeBlock :code="multiArgsRetsExample" />

### `rets`

This field determines the data that is sent back to the client from the server. It can be any [Zap type](./types.md).

- If the server doesn't return any data, the `rets` field should be excluded.
- The function can return multiple values by separating each type with a comma and wrapping them all in parentheses:

<CodeBlock :code="multiArgsRetsExample" />
