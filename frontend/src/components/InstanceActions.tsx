import styled from "styled-components";
import {InstanceDescriptor} from "../model";
import Btn from "./Button";
import { gql, useMutation } from "@apollo/client";

type Props = {
    name: string,
    deselect: () => void,
    state: string
}

const InstanceActions = ({name, state, deselect}: Props) => {

    const [remove] = useMutation(gql`
        mutation Mutation($name: String!,$password: String!) {
            deleteServer(name: $name,password: $password)
        }
    `);

    const [ctl] = useMutation(gql`
        mutation Mutation($name: String!,$shouldRun: Boolean!) {
            shouldRun(name: $name,shouldRun: $shouldRun)
        }
    `);

    const onClickDelete = () => {
        let password = prompt("Please enter the password to delete this server");

        if (!password) {
            return;
        }
        
        deselect();
        remove({
            variables: {
                name,
                password
            }
        })
    };

    const switchServer = (shouldRun: boolean) => () => {
        deselect()
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

    const rconOnClick = () => {
        window.location.href = `/rcon?name=${name}`;
    };

    return <>
        {(state == "Running")
            ? <>
                <Btn onClick={switchServer(false)}>Stop</Btn><br />
                <Btn onClick={rconOnClick}>Rcon</Btn><br />
            </>
            : null
        }
        {(state !== "Running")
            ? <>
                <Btn onClick={switchServer(true)}>Start</Btn><br />
                <Btn onClick={alterOnClick}>Alter</Btn><br />
              </>
            : null
        }
        <Btn onClick={onClickDelete}>Delete</Btn><br />
    </>
}


export default InstanceActions;