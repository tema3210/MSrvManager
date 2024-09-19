import styled from "styled-components";

type Props = {
    onClick?: () => void
} & React.PropsWithChildren;

const Inner = styled.button`
    
`;

const Btn = ({onClick,children}: Props) => {
    return <Inner onClick={onClick ?? (()=>{}) }>{children}</Inner>
};

export default Btn;