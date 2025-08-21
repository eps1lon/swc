let foo = ({ aaa: a, bbb: b, ccc: c }) => {
    c(1);
};
foo({ aaa: 1, bbb: 2, ccc: console.log });