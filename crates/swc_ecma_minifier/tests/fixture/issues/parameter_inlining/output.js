function complex(foo) {
    const fn = undefined;
    if (Math.random() > .5) throw new Error();
    return fn?.(foo);
}
console.log(complex("foo")), console.log(complex("bar")), console.log(complex("baz"));
function test1(a) {
    const b = undefined;
    return console.log(a, b), a + b;
}
test1("x"), test1("y"), test1("z");
function test2() {
    const x = 5, y = 10;
    return 2 * x + y;
}
console.log(test2()), console.log(test2()), console.log(test2());
function test3(a) {
    const b = "same", c = !0;
    return a + b + c;
}
test3(1), test3(2), test3(3);
function test4(x) {
    return x += 1;
}
test4(void 0), test4(void 0);
function test5(a, b) {
    return console.log(arguments.length), a + b;
}
test5(1), test5(1);
const fn = function(x) {
    const y = 5;
    return x + y;
};
fn(null), fn(null);
const arrow = (a, b)=>a + b;
arrow(void 0, 3), arrow(void 0, 3);
function withSideEffects(val) {
    const cb = undefined;
    return console.log("side effect"), cb ? cb(val) : val;
}
withSideEffects("test"), withSideEffects("test2");