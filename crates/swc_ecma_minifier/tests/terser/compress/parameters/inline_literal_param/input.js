function processWithCallback(data, callback) {
  if (typeof callback === "function") {
    return callback(data);
  }
  return data;
}

console.log(processWithCallback("hello", null));
console.log(processWithCallback("world", null));
console.log(processWithCallback("test", null));