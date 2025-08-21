let foo = ({ ccc }) => {
    ccc(1);
};
foo({ aaa: 1, bbb: 2, ccc: console.log });