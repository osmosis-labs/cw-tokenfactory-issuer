import {
  Box,
  Center,
  Heading,
  Tab,
  Table,
  TableContainer,
  TabList,
  TabPanel,
  TabPanels,
  Tabs,
  Tbody,
  Td,
  Tr,
  VStack,
} from "@chakra-ui/react";
import type { NextPage } from "next";
import Head from "next/head";
import { useDenom, useOwner } from "../api/tokenfactoryIssuer";
import Blacklistng from "../components/blacklisting";
import Minting from "../components/minting";

const Home: NextPage = () => {
  const { data: denomRes, error: denomErr } = useDenom();
  const { data: ownerRes, error: ownerErr } = useOwner();

  return (
    <Center my="10" minWidth="container.xl">
      <Head>
        <title>Tokenfactory Issuer UI</title>
        <meta name="description" content="Tokenfactory Issuer UI" />
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <VStack maxW="container.xl" spacing={10} align="stretch">
        <Box>
          <Heading my="10" as="h2" size="lg">
            General Info
          </Heading>
          <TableContainer>
            <Table variant="simple">
              <Tbody>
                <Tr>
                  <Td>denom</Td>
                  <Td>{denomRes?.denom}</Td>
                </Tr>
                <Tr>
                  <Td>issuer contract owner</Td>
                  <Td>{ownerRes?.address}</Td>
                </Tr>
              </Tbody>
            </Table>
          </TableContainer>
        </Box>
        <Tabs variant="soft-rounded" colorScheme="green">
          <TabList>
            <Tab>Minting</Tab>
            <Tab>Blacklisting</Tab>
            <Tab>Burning</Tab>
          </TabList>
          <TabPanels>
            <TabPanel>
              <Minting></Minting>
            </TabPanel>
            <TabPanel>
              <Blacklistng></Blacklistng>
            </TabPanel>
            <TabPanel>
              <p>burn</p>
            </TabPanel>
          </TabPanels>
        </Tabs>
      </VStack>
    </Center>
  );
};

export default Home;
