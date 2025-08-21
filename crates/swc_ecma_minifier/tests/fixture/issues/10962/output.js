export default function render(factory) {
    return factory(({ ccc }) => {
        ccc(1);
    }, [
        0,
        19
    ]);
}