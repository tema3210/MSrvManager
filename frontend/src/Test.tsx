
const Component = ({f, obj}: {f: () => void, obj: {}}) => {
    return <>
        <button onClick={() => f()}>It calls function prop</button>
        <p>Object: {JSON.stringify(obj)}</p>
    </>
}

export default Component;