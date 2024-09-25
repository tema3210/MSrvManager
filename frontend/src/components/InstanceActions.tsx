import styled from "styled-components";
import {InstanceDescriptor} from "../model";
import Btn from "./Button";
import { gql, useMutation } from "@apollo/client";

type Props = {
    instance: InstanceDescriptor,
    deselect: () => void
}

const InstanceActions = ({instance,deselect}: Props) => {
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

    const onClickDelete = () => {
        deselect();
        remove({
            variables: {
                name
            }
        })
    };

    const switchServer = (shouldRun: boolean) => () => {
        console.log(name,shouldRun)
        ctl({
            variables: {
                name,
                shouldRun
            }
        })
    };

    const alterOnClick = () => {
        window.location.href = `/alter?name=${name}`;
    };

    return <div>
        {(state == "Running")
            ? <Btn onClick={switchServer(false)}>Stop</Btn>
            : <Btn onClick={switchServer(true)}>Start</Btn>
        }<br />
        {(state !== "Running")
            ? <><Btn onClick={alterOnClick}>Alter</Btn><br /></> 
            : null
        }
        <Btn onClick={onClickDelete}>Delete</Btn><br />
    </div>
}


export default InstanceActions;