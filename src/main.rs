use clap::{arg, command, Parser};
use clap_num::number_range;
use ark_poly::{univariate::DensePolynomial, EvaluationDomain, GeneralEvaluationDomain};
use ark_bn254::{Bn254, Fr, G1Projective};
use ark_ec::AffineCurve;
use ark_ff::bytes::FromBytes;
use quotient_pir::{
    primitives::{
        compute_lagrange_basis_commitments, compute_lagrange_evaluation_proofs, compute_qs,
    },
    serializer::Serializer,
    server::Config,
    update_keys::UpdateKey,
};

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidSk,
    InvalidLog2Capacity,
    InvalidIdNulOrTrap,
}

fn log_2_capacity_range(s: &str) -> Result<u8, String> {
    number_range(s, 10, 28)
}

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "config")]
#[command(about = "Generates a config file for quotient-pir", long_about = None)]
struct Cli {
    /// The powers of tau (PTAU) file containing a phase 1 trusted setup output
    #[arg(short, long, required = true,)]
    ptau: String,

    /// The capacity of the accumulator expressed in log_2 (e.g. log_2(1024) = 10)
    #[arg(short, long, required = false, default_value = "10", value_parser=log_2_capacity_range)]
    log_2_capacity: u8,

    /// The path to the output file
    #[arg(short, long, required = true,)]
    output: String,
}

fn main() {
    let args = Cli::parse();
    let table_size: usize = 2u64.pow(args.log_2_capacity as u32) as usize;
    let domain = GeneralEvaluationDomain::<Fr>::new(table_size).unwrap();
    let num_g1_points = table_size;
    let num_g2_points = 0; //table_size;
    let (srs, _srs_g2) =
        ppot_rs::ptau::read(&args.ptau, num_g1_points, num_g2_points).unwrap();

    let srs_proj: Vec<G1Projective> = srs.iter().map(|t| t.into_projective()).collect();

    let zh: DensePolynomial<Fr> = domain.vanishing_polynomial().into();

    let zero_hex = "251a679ce76f71008e8f811649361985e499a17da6411eef0ba206cd72b3771f";
    let zero_bytes = hex::decode(zero_hex).unwrap();
    let zero_value = Fr::read(zero_bytes.as_slice()).unwrap();

    let lagrange_basis_commitments = compute_lagrange_basis_commitments(&srs);

    let vanishing_opening_cms = compute_qs::<Bn254>(&zh, &domain, &srs_proj);
    let lagrange_opening_cms = compute_lagrange_evaluation_proofs::<Bn254>(&srs, &domain);

    let up_keys: Vec<_> = (0..domain.size())
        .map(|i| UpdateKey::<Bn254>::new(i, &lagrange_opening_cms, &vanishing_opening_cms))
        .collect();

    let config = Config {
        zero_value,
        domain,
        lagrange_basis_commitments: lagrange_basis_commitments.clone(),
        up_keys,
    };
    config.write_to_path(&args.output).unwrap();
}
