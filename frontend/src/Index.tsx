import { ChangeEventHandler, useState } from "react";
import { makeOnLoad } from "./lib";
import { InstanceDescriptor } from "./model";
import InstanceDisplay from "./components/InstanceDesc";

import { gql, useMutation, useSubscription, useQuery } from "@apollo/client";
import styled from "styled-components";

const InstanceWrapper = styled.div`
    margin-left: 2rem;
    width: calc(100% - 2rem);
`;

const TextBig = styled.div`
    font-size: 2rem;
    color: #db9f30;
    margin-left: 2rem;
    width: calc(100% - 2rem);
`;

const Footer = styled.div`
    position: absolute;
    bottom: 0;
    height: 3rem;
`;

const Index = ({}) => {
    const { data, loading, error } = useSubscription<{servers: InstanceDescriptor[]}>(gql`
        subscription {
            servers {
                name,
                memory,
                state,
                maxMemory,
                mods,
                port
            }
        }
    `);

    const { data: AVdata, loading: AVloading } = useQuery(gql`
        {
            appVersion
        }
    `);

    const [vars,setVars] = useState<object | null>();

    const [createServer] = useMutation(gql`
        mutation Mutation($file: Upload!) {
            newServer(data: {
                name: "test",
                upCmd: "echo hi",
                url: "http://google.com",
                maxMemory: 1.5,
                port: 25565,
                rcon: 26001,
                instanceUpload: $file
            })
        }
        `,
        {
            variables: vars ?? {}
        }
    );

    if (loading) return "Loading server list";
    if (error) return <pre>{error.message}</pre>

    const onChange: ChangeEventHandler<HTMLInputElement> = (ev) => {
        let file = ev.target.files?.[0];

        if (file) {
            setVars({
                file
            })
        } else {
            setVars(null)
        }
    };

    return <>
        <TextBig>We have these servers:</TextBig>
        <InstanceWrapper>
            {(data?.servers ?? []).map((v) => (<InstanceDisplay instance={v} key={v.name}/>))}
        </InstanceWrapper>
        <input type="file" onChange={onChange} /><br />
        <button onClick={() => createServer()}>test create server (with file)</button>
        <Footer><TextBig>Version: {(AVloading)? AVdata.appVersion : "-"}</TextBig></Footer>
    </>
}

window.onload = makeOnLoad(<Index />)
