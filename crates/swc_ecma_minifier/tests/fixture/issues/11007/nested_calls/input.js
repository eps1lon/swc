const wrapper = (_s, fn) => {
    return fn();
};

wrapper("test", () => foo(bar(baz())));