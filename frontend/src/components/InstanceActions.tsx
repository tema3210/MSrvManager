import styled from "styled-components";
import {InstanceDescriptor} from "../model";
import Btn from "./Button"
import { gql, useMutation } from "@apollo/client";


type Props = {
    instance: InstanceDescriptor, 
}

const InstanceActions = ({instance}: Props) => {
    const {name,state} = instance;

    const [remove] = useMutation(gql`
        mutation Mutation($name: String!) {
            deleteServer(name: $name)
        }
    `);

    const [ctl] = useMutation(gql`
        mutation Mutation($name: String!,$shouldRun: Boolean!) {
            shouldRun(name: $name,shouldRun: $shouldRun)
        }
    `);

    const onClickDelete = () => remove({
        variables: {
            name
        }
    });

    const switchServer = (shouldRun: boolean) => () => ctl({
        variables: {
            name,
            shouldRun
        }
    });

    const alterOnClick = () => {
        window.location.href = `/alter?name=${name}`;
    };

    return <div>
        {(state == "RUNNING")
            ? <Btn onClick={switchServer(false)}>Stop</Btn> 
            : <Btn onClick={switchServer(true)}>Start</Btn>
        }<br />
        <Btn onClick={alterOnClick}>Alter</Btn><br />
        <Btn onClick={onClickDelete}>Delete</Btn><br />
    </div>
}


export default InstanceActions;