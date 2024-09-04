import styled from "styled-components";

const LeBalique = styled.div`
    color: #1dcfa9;
    font-size: 2rem;
`;

const Component = ({name, age, flag, array}: {name: string,age: number, flag: boolean, array: any[] }) => {
    return <>
        <LeBalique>Hello {name} you're {age}</LeBalique>
        { (flag) ? <div>the flag is set</div> : <></>}
        Array:
        <ul>
            {array.map((v) => <li>{v}</li>)}
        </ul>
        <c-test obj={"{\"b\": 1}"} name={name}></c-test>
    </>
}

export type ComponentProps = Parameters<typeof Component>[0];

export default Component;