const wrapper = (_s, fn) => {
    return fn();
};

wrapper("test1", () => foo(...args));
wrapper("test2", () => bar(a, ...rest, b));