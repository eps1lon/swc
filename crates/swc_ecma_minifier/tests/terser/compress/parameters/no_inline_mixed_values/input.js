function process(value, debug) {
  if (debug) {
    console.log("Processing:", value);
  }
  return value * 2;
}

console.log(process(5, true));
console.log(process(10, false));
console.log(process(15, true));