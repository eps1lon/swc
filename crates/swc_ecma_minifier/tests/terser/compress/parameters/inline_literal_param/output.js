function processWithCallback(data) {
  const callback = null;
  if (typeof callback === "function") {
    return callback(data);
  }
  return data;
}

console.log(processWithCallback("hello"));
console.log(processWithCallback("world"));
console.log(processWithCallback("test"));