import styled from "styled-components";
import {InstanceDescriptor} from "../model";

const Inner = styled.div<{selected: boolean}>`
    background-color: ${(p) => p.selected ? '#ffffffbc' : '#ffffff5c'};
    padding: 16px;
    border-radius: 8px;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
    cursor: pointer;
    transition: background-color 0.3s;
`;

const Title = styled.h2`
    margin: 0;
    font-size: 1.5em;
    color: #333;
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

const Desc = ({instance, selected, setSelected}: Props) => {
    const {name, state, memory, maxMemory, port} = instance;

    return (
        <Inner onClick={setSelected} selected={selected}>
            <Title>{name}</Title>
            <Info>
                <InfoItem>State: {state}</InfoItem>
                <InfoItem>Memory usage: {memory ?? 'N/A'}</InfoItem>
                <InfoItem>Max memory: {maxMemory} GB</InfoItem>
                <InfoItem>Port: {port}</InfoItem>
            </Info>
        </Inner>
    );
}

export default Desc;