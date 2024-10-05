import { makeOnLoad, SSRProps } from "./lib";
import styled from 'styled-components';
import { gql, useMutation, useSubscription } from '@apollo/client';
import { useState } from "react";

const Form = styled.form`
    display: flex;
    flex-direction: column;
    gap: 1rem;
    max-width: 400px;
    margin: 0 auto;
`;

const FormGroup = styled.div`
    display: flex;
    flex-direction: column;
`;

const Label = styled.label`
    margin-bottom: 0.5rem;
    font-weight: bold;
`;

const Input = styled.input`
    padding: 0.5rem;
    font-size: 1rem;
    border: 1px solid #ccc;
    border-radius: 4px;
`;

const Button = styled.button`
    padding: 0.75rem;
    font-size: 1rem;
    color: #fff;
    background-color: #d3af6c;
    border: none;
    border-radius: 4px;
    cursor: pointer;

    &:hover {
        background-color: #db9f30;
    }
`;

const OutputContainer = styled.div`
    margin-top: 2rem;
`;

const OutputTitle = styled.h3`
    margin-bottom: 1rem;
`;

const OutputPre = styled.pre`
    padding: 1rem;
    background-color: #f8f9fa;
    border: 1px solid #db9f30;
    border-radius: 4px;
`;

const Rcon = ({ pageData }: SSRProps) => {

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

    if (error) return <pre>Error: {error.message}</pre>;

    const handleSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        const formData = new FormData(event.currentTarget);
        const message = formData.get('message') as string;

        await msg({ variables: { name: pageData.name, message, password } });
    };

    return (
        <div>
            <Form onSubmit={handleSubmit}>
                <FormGroup>
                    <Label htmlFor="message">Message:</Label>
                    <Input type="text" id="message" name="message" required />
                </FormGroup>
                <Button type="submit">Send</Button>
            </Form>
            <OutputContainer>
                <OutputTitle>Rcon Output:</OutputTitle>
                <OutputPre>{data?.rconOutput}</OutputPre>
            </OutputContainer>
        </div>
    );
}

window.onload = makeOnLoad(Rcon);