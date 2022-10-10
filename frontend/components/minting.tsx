import {
  Box,
  Button,
  Divider,
  FormControl,
  FormHelperText,
  FormLabel,
  Heading,
  Input,
  Table,
  TableContainer,
  Tbody,
  Td,
  Th,
  Tr,
  VStack,
} from "@chakra-ui/react";
import { useForm } from "react-hook-form";
import {
  getTokenIssuerSigningClient,
  useDenom,
  useMintAllowances,
} from "../api/tokenfactoryIssuer";

const Minting = () => {
  const { data: denomRes } = useDenom();
  return (
    <Box>
      <Allowances></Allowances>
      <VStack>
        <SetMinterForm denom={denomRes?.denom || ""}></SetMinterForm>
        <MintForm denom={denomRes?.denom || ""}></MintForm>
      </VStack>
    </Box>
  );
};

const Allowances = () => {
  const { data: mintAllowancesRes } = useMintAllowances();
  return (
    <>
      <Heading my="10" as="h2" size="lg">
        Minting
      </Heading>
      <Heading my="5" as="h3" size="md">
        Allowances
      </Heading>
      <TableContainer>
        <Table variant="simple">
          <Tbody>
            <Tr>
              <Th>address</Th>
              <Th>allowance</Th>
            </Tr>
            {mintAllowancesRes?.allowances.map((allowance) => {
              return (
                <Tr key={"mint_allowance_" + allowance.address}>
                  <Td>{allowance.address}</Td>
                  <Td>{allowance.allowance}</Td>
                </Tr>
              );
            })}
          </Tbody>
        </Table>
      </TableContainer>
    </>
  );
};
const SetMinterForm = ({ denom }: { denom: string }) => {
  const { mutate } = useMintAllowances();
  const {
    handleSubmit,
    register,
    reset,
    formState: { errors, isSubmitting },
  } = useForm();
  const onSubmit = async (values) => {
    const client = await getTokenIssuerSigningClient();
    await client.setMinter(values);
    mutate();
    reset();
  };
  return (
    <Box
      my="10"
      border="2px"
      borderColor="gray.200"
      borderRadius="md"
      p="9"
      minWidth="container.md"
    >
      <Heading my="5" as="h3" size="md">
        Set Allowances
      </Heading>
      <Divider />
      <form onSubmit={handleSubmit(onSubmit)}>
        <FormControl isRequired my="5">
          <FormLabel>Mint allowance</FormLabel>
          <Input
            type="number"
            id="allowance"
            disabled={isSubmitting}
            {...register("allowance")}
          ></Input>
          <FormHelperText>
            amount of `{denom}` to allow minter to mint
          </FormHelperText>
        </FormControl>
        <FormControl isRequired my="5">
          <FormLabel>address</FormLabel>
          <Input
            type="text"
            id="address"
            disabled={isSubmitting}
            {...register("address")}
          />
          <FormHelperText>minter address</FormHelperText>
        </FormControl>
        <Button
          mt={4}
          colorScheme="teal"
          isLoading={isSubmitting}
          type="submit"
        >
          Set Minter
        </Button>
      </form>
    </Box>
  );
};

const MintForm = ({ denom }: { denom: string }) => {
  const { mutate } = useMintAllowances();
  const {
    handleSubmit,
    register,
    reset,
    formState: { errors, isSubmitting },
  } = useForm();
  const onSubmit = async (values) => {
    const client = await getTokenIssuerSigningClient();
    await client.mint(values);
    mutate();
    reset();
  };
  return (
    <Box
      my="10"
      border="2px"
      borderColor="gray.200"
      borderRadius="md"
      p="9"
      minWidth="container.md"
    >
      <Heading my="5" as="h3" size="md">
        Mint
      </Heading>
      <Divider />
      <form onSubmit={handleSubmit(onSubmit)}>
        <FormControl isRequired my="5">
          <FormLabel>mint amount</FormLabel>
          <Input
            type="number"
            id="amount"
            disabled={isSubmitting}
            {...register("amount")}
          ></Input>
          <FormHelperText>amount of `{denom}` to be minted</FormHelperText>
        </FormControl>
        <FormControl isRequired my="5">
          <FormLabel>to address</FormLabel>
          <Input
            type="text"
            id="toAddress"
            disabled={isSubmitting}
            {...register("toAddress")}
          />
          <FormHelperText> address to be minted to </FormHelperText>
        </FormControl>
        <Button
          mt={4}
          colorScheme="teal"
          isLoading={isSubmitting}
          type="submit"
        >
          Mint
        </Button>
      </form>
    </Box>
  );
};

export default Minting;
