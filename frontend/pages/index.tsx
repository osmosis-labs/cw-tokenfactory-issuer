import type { NextPage } from "next";
import Head from "next/head";
import { useState } from "react";
import { useDenom } from "../api/counter";
import styles from "../styles/Home.module.css";

const Home: NextPage = () => {
  const { denom, error } = useDenom();
  const [isLoading, setLoading] = useState(false);

  return (
    <div className={styles.container}>
      <Head>
        <title>Tokenfactory Issuer UI</title>
        <meta name="description" content="Tokenfactory Issuer UI" />
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <main className={styles.main}>
        <h1 className={styles.title}>Denom</h1>

        <p
          className={
            isLoading ? [styles.count, styles.pulse].join(" ") : styles.count
          }
        >
          {denom === undefined ? "?" : denom}
        </p>

        {error ? <p className={styles.error}>Error: {error.message}</p> : <></>}
        {/* 
        <div className={styles.grid}>
          <a
            className={styles.card}
            onClick={async () => {
              setLoading(true);
              await increase();
              setLoading(false);
            }}
          >
            <h2>＋ Increase Counter</h2>
          </a>
        </div> */}
      </main>
    </div>
  );
};

export default Home;
