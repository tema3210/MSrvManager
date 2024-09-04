
const Component = ({f: fn, obj, name}: {f: () => void, obj: {}, name: string }) => {
    return <>
        <button onClick={() => fn()}>It calls function prop of {name}</button>
        <p>Object: {JSON.stringify(obj)}</p>

    </>
}

export type ComponentProps = Parameters<typeof Component>[0];

export default Component;