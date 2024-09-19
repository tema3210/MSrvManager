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

    const { data: AVdata, loading: AVloading } = useQuery(gql`
        query {
            appVersion
        }
    `);

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

    

    const [selected,setSelected] = useState<InstanceDescriptor | null>(null);

    if (loading) return "Loading server list";
    if (error) return <pre>{error.message}</pre>

    //TODO: make a table.
    return <>
        <TextBig>We have these servers:</TextBig>
        <InstanceWrapper>
            {(data?.servers ?? [])
                .map((v) => (
                    <InstanceDisplay 
                        instance={v} 
                        key={v.name} 
                        selected={selected?.name === v.name}
                        setSelected={
                            (selected?.name === v.name)
                                ? () => setSelected(null) 
                                : () => setSelected(v)
                        }
                    />
                ))
            }
        </InstanceWrapper>
        <p>SELECTED: {JSON.stringify(selected)}</p>
        <Footer><TextBig>Version: {(AVloading)? "-" : AVdata.appVersion }</TextBig></Footer>
    </>
}

window.onload = makeOnLoad(<Index />)
