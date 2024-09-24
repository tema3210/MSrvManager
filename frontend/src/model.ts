export type ServerState = "Running" | "Stopped" | "Crashed"

export type InstanceDescriptor = {
    name: string,
    mods: string,
    state: ServerState,

    memory: number | null,
    maxMemory: number,
    port: number
}

export type PortsInfo = {
    portsTaken: {
        ports: number[],
        rcons: number[],
        portLimits: [number,number],
        rconLimits: [number,number]
    }
}