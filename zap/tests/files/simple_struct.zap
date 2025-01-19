event MyEvent = {
    from: Server,
    type: Reliable,
    call: ManyAsync,
    data: struct {
        foo: string,
        bar: u8,
    }
}
