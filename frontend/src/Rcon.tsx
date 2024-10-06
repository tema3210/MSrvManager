import { makeOnLoad, SSRProps } from "./lib";
import styled from 'styled-components';
import { gql, useMutation, useSubscription } from '@apollo/client';
import { useState } from "react";
import { HomeLink, Label, SInput, TextBig } from "./components/UIComps";
import Btn from "./components/Button";

const Form = styled.div`
    display: flex;
    flex-direction: row;
    align-items: stretch;
    gap: 1rem;
    height: 12vh;
    margin: 0 auto;
`;

const VStack = styled.span`
    height: 4.5rem;
    display: inline-flex;
    align-items: start;
    justify-content: stretch;
    flex-direction: column;
`;

const OutputContainer = styled.div`
    margin-top: 1rem;
    height: calc(100vh - 12vh - 2rem);
    margin-bottom: 1rem;
`;

const OutputPre = styled.pre`
    padding: 1rem;
    background-color: #f8f9fa;
    height: 100%;
    border: 1px solid #db9f30;
    border-radius: 4px;
`;

type Props = {
    name: string
}

const Rcon = ({ pageData }: SSRProps<Props>) => {

    let [password] = useState(() => prompt("Please enter the password to use rcon on this server"));

    if (!password) {
        window.location.href = '/';
        return;
    }

    const [msg] = useMutation(gql`
        mutation Mutation($name: String!,$message: String!,$password: String!) {
            rconMessage(name: $name,message: $message,password: $password)
        }
    `);

    const { error, data } = useSubscription(
        gql`
            subscription Subscription($name: String!) {
                rconOutput(name: $name)
            }
        `, 
        {
            variables: {
                name: pageData.name
            }
        }
    );

    const [message,setMsg] = useState<string>("");

    if (error) {
        window.location.href = '/';
        return null;
    };

    // for sanity this must only be primitive string or null
    let rconOutput: string[] = data?.rconOutput ?? [];

    return (
        <div>
            <Form>
                <VStack>
                    <HomeLink href="/">Home</HomeLink>
                    <TextBig>Rcon of {pageData.name} server: </TextBig>
                </VStack>
                <VStack>
                    <Label>Command:</Label>
                    <SInput type="text" value={message} onChange={(ev) => setMsg(ev.target.value)} />
                </VStack>
                <VStack><Btn onClick={() => { msg({ variables: { name: pageData.name, message, password } }); }}>Send</Btn></VStack>
            </Form>
            <OutputContainer>
                <TextBig>Rcon Output:</TextBig>
                <OutputPre>{
                    rconOutput.map((line) => (<p>{line}</p>))
                }</OutputPre>
            </OutputContainer>
        </div>
    );
}

window.onload = makeOnLoad(Rcon);