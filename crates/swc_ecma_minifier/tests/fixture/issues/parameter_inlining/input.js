// Test case from issue #10931
function complex(foo, fn) {
  // prevent inlining
  if (Math.random() > 0.5) throw new Error()
  return fn?.(foo)
}

console.log(complex("foo"))
console.log(complex("bar"))
console.log(complex("baz"))

// Test various parameter inlining scenarios

// All callsites pass undefined
function test1(a, b) {
  console.log(a, b);
  return a + b;
}
test1("x");
test1("y");
test1("z");

// All callsites pass literal values
function test2(x, y) {
  return x * 2 + y;
}
console.log(test2(5, 10));
console.log(test2(5, 10));
console.log(test2(5, 10));

// Mixed consistent/inconsistent parameters
function test3(a, b, c) {
  return a + b + c;
}
test3(1, "same", true);
test3(2, "same", true); 
test3(3, "same", true);

// Parameter is mutated - should not inline
function test4(x) {
  x = x + 1;
  return x;
}
test4(undefined);
test4(undefined);

// Uses arguments object - should not inline
function test5(a, b) {
  console.log(arguments.length);
  return a + b;
}
test5(1);
test5(1);

// Named function expression
const fn = function named(x, y) {
  return x + y;
};
fn(null, 5);
fn(null, 5);

// Arrow functions are not handled yet
const arrow = (a, b) => a + b;
arrow(undefined, 3);
arrow(undefined, 3);

// Function with side effects - still safe to inline params
function withSideEffects(val, cb) {
  console.log("side effect");
  return cb ? cb(val) : val;
}
withSideEffects("test", undefined);
withSideEffects("test2", undefined);