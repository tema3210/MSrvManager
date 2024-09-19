import styled from "styled-components";
import {InstanceDescriptor} from "../model";
import Btn from "./Button"


type Props = {
    instance: InstanceDescriptor, 
}

const InstanceActions = ({instance}: Props) => {
    const {name,state} = instance;

    return <div>
        <Btn>Delete</Btn><br />
        <Btn>Start/Stop</Btn><br />
        <Btn></Btn><br />
    </div>
}


export default InstanceActions;