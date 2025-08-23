const wrapper = (_s, fn) => {
    return fn();
};

wrapper("test1", () => obj.method());
wrapper("test2", () => obj.nested.method(arg));