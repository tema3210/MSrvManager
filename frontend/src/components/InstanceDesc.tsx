import styled from "styled-components";
import {InstanceDescriptor} from "../model";

const Inner = styled.div<{selected: boolean}>`
    background-color: ${(p) => p.selected ? '#ff9034' : 'rgba(255, 255, 255, 0.9)'};
    padding: 16px;
    border-radius: 8px;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
    transition: background-color 0.3s;
    margin-bottom: 16px;
`;

const Title = styled.h2`
    margin: 0;
    font-size: 1.5em;
    color: #333;
    user-select: none;
    cursor: pointer;
`;

const Info = styled.div`
    display: flex;
    flex-direction: column;
    margin-top: 8px;
`;

const InfoItem = styled.div`
    margin: 4px 0;
    font-size: 1em;
    color: #666;
`;

type Props = {
    instance: InstanceDescriptor, 
    selected: boolean,
    setSelected: () => void
}

const RTInner = styled.svg`
    width: 1.5rem;
    height: 1.5rem;
    margin-right: 0.5rem;
`;

const RightTriangle = () => (<RTInner viewBox="0 0 100 100" >
    <polygon points="0,0 0,100 100,50" fill="#4e812b" />
</RTInner>)


const Desc = ({instance, selected, setSelected}: Props) => {
    const {name, state, memory, max_memory, port} = instance;

    return (
        <Inner selected={selected}>
            <Title onClick={setSelected}><RightTriangle />{name}</Title>
            <Info>
                <InfoItem>State: {state}</InfoItem>
                <InfoItem>Memory usage: {memory ?? 'N/A'}</InfoItem>
                <InfoItem>Max memory: {max_memory} GB</InfoItem>
                <InfoItem>Port: {port}</InfoItem>
                <InfoItem>Mods URL: <a href={instance.mods}>{instance.mods}</a></InfoItem>
            </Info>
        </Inner>
    );
}

export default Desc;