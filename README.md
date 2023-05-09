# iamthat - a partial reimplementation of AWS IAM policy evaluation

The goal of this crate is to reimplement AWS IAM policy evaluation logic in a
self-contained Rust crate that can run offline without talking to AWS. On that
core, we can build tools that, for example, run unit tests for IAM policies,
asserting that some actions are allowed and others are denied.

This crate builds both a Rust library and a command-line tool.

## Command-line usage

Check one request against some resource policies:

    iamthat eval --request-file <request.json> --resource-policy-file <policy.json>

## Features

Policy types:

- [x] Parse AWS IAM policy JSON.
- [ ] Resource policies.
- [ ] Identity policies.
- [ ] Service control policies.
- [ ] Session policies.

Testing:

- [ ] JSON "scenario" files to the tree containing both a request and a
  series of policies, and an assertion that the requests are allowed or
  denied.
- [ ] Automatically test against access analyzer, for cases that are supported
  by both.

Policy evaluation:

- [x] Check a single action name, with no parameters, against a policy.
- [ ] Check resource name.
- [ ] Check condition keys.
- [ ] NotAction, NotResource, etc.
- [ ] Lint a policy for common errors.
- [ ] If the action is denied, say which policy and statement caused the
  denial.

AWS API integration:

- [ ] Download relevant policies from AWS?

## IAM Policy Simulator

The function and purpose of this crate significantly overlaps with
[AWS IAM Policy Simulator][policy_sim]. The main differences are:

Policy Simulator is much more complete and backed by AWS. It requires online
access to AWS, and has some limitations on which policies it can evaluate.

This crate is open source and can be run offline, but is currently extremely
incomplete.

[policy_sim]: https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies_testing-policies.html

## Status

This crate is very incomplete and experimental. It is not ready for use.

The AWS IAM policy evaluation logic is very complicated and not perfectly clearly
documented. Even for features that are marked as implemented there is a
significant likelihood that some evaluations will be inconsistent with AWS's
behavior.
