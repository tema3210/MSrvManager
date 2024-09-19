import styled from "styled-components";
import {InstanceDescriptor} from "../model";

const Inner = styled.div<{selected: boolean}>`
    background-color: ${(p) => p.selected ? '#ffffffbc' : '#ffffff5c'};
`;

type Props = {
    instance: InstanceDescriptor, 
    selected: boolean,
    setSelected: () => void
}

const Desc = ({instance, selected, setSelected}: Props) => {
    const {name,state} = instance;

    return <Inner onClick={setSelected} selected={selected}>
        {name} is {state};
        Memory usage: {instance.memory ?? null};
        Max memory: {instance.maxMemory} GB;
        At: _:{instance.port};
    </Inner>
}


export default Desc;