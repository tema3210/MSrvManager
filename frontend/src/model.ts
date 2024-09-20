export type ServerState = "RUNNING" | "STOPPED" | "CRASHED"

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
        port_limits: [number,number],
        rcon_limits: [number,number]
    }
}