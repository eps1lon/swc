export default function render(factory) {
  return factory(({aaa, bbb, ccc}) => {
    ccc(1)
  }, [0, 19])
}