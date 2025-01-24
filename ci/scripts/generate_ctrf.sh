#!/bin/bash

output_file="records/test_results.json"
output=""
total_passed=0
total_failed=0
total_skipped=0   
total_global_passed=0
total_global_failed=0
total_global_skipped=0 

test_suites=(
    "UniswapV2ERC20"
    "UniswapV2Factory"
    "UniswapV2Pair"
    "AccessControl"
    "Ownable"
    "Ownable2Step"
    "AccessControlDefaultAdminRules"
    "AccessControlEnumerable"
    "AccessManaged"
    "AccessManager"
    "AuthorityUtils"
    "VestingWallet"
    "VestingWalletCliff"
    "Governor"
    "TimelockController"
    "GovernorCountingFractional"
    "GovernorERC721"
    "GovernorPreventLateQuorum"
    "GovernorStorage"
    "GovernorTimelockAccess"
    "GovernorTimelockCompound"
    "GovernorTimelockControl"
    "GovernorVotesQuorumFraction"
    "GovernorWithParams"
    "Votes"
    "ERC2771Context"
    "ERC2771Forwarder"
    "Clones"
    "ERC1967Proxy"
    "ERC1967Utils"
    "BeaconProxy"
    "UpgradeableBeacon"
    "ProxyAdmin"
    "TransparentUpgradeableProxy"
    "Initializable"
    "UUPSUpgradeable"
    "Environment sanity"
    "ERC1155"
    "ERC1155Burnable"
    "ERC1155Pausable"
    "ERC1155Supply"
    "ERC1155URIStorage"
    "ERC1155Holder"
    "ERC1155Utils"
    "ERC20"
    "ERC1363"
    "ERC20Burnable"
    "ERC20Capped"
    "ERC20FlashMint"
    "ERC20Pausable"
    "ERC20Permit"
    "ERC20Votes"
    "ERC20Wrapper"
    "ERC4626"
    "ERC20TemporaryApproval"
    "SafeERC20"
    "ERC721"
    "ERC721Burnable"
    "ERC721Consecutive"
    "ERC721Pausable"
    "ERC721Royalty"
    "ERC721URIStorage"
    "ERC721Votes"
    "ERC721Wrapper"
    "ERC721Holder"
    "ERC721Utils"
    "Address"
    "Arrays"
    "Strings"
    "Context"
    "Create2"
    "Multicall"
    "Nonces"
    "Packing"
    "Panic"
    "Pausable"
    "ReentrancyGuard"
    "ReentrancyTransientGuard"
    "ShortStrings"
    "SlotDerivation"
    "StorageSlot"
    "ECDSA"
    "EIP712"
    "MerkleProof"
    "MessageHashUtils"
    "P256"
    "RSA"
    "SignatureChecker"
    "ERC165Checker"
    "Math"
    "SafeCast"
    "SignedMath"
    "BitMap"
    "Checkpoints"
    "CircularBuffer"
    "DoubleEndedQueue"
    "EnumerableMap"
    "EnumerableSet"
    "Heap"
    "MerkleTree"
    "Time"
)

pattern=$(IFS='|'; echo "${test_suites[*]}")

start_time=$(date +%s)

for input_file in records/*.txt; do
    current_suite=""
    current_subsuite=""
    case=$(basename "${input_file%.txt}")

    if [ ! -s "$input_file" ]; then
        echo "File $input_file is empty. Skipping..."
        total_skipped=1
        total_global_skipped=$((total_global_skipped + 1))
        continue
    fi

    tests_found=false

    if [[ $input_file == *tests* ]]; then
        while IFS= read -r line; do
            if [[ "$line" == *" ... ok" ]]; then
                total_global_passed=$((total_global_passed + 1))
                tests_found=true
            fi

            if [[ "$line" == *" ... FAILED" ]]; then
                total_global_failed=$((total_global_failed + 1))
                tests_found=true
            fi

            if [[ ! $line =~ ^test[[:space:]][^[:space:]]+::case_[0-9]+_[^[:space:]]+[[:space:]]+\.\.\.[[:space:]]+(ok|FAILED)$ ]]; then
                continue
            fi

            # Extract the status
            result=$(echo "$line" | awk '{print $NF}')
            status=""
            if [[ $result == "ok" ]]; then
                status="passed"
            elif [[ $result == "FAILED" ]]; then
                status="failed"
            else
                status="unknown"
            fi

            # Extract the deploy
            deploy=$(echo "$line" | sed -n 's/^test \([^:]*\)::case_[0-9]*_.*/\1/p')

            # Extract the contract
            contract=$(echo "$line" | awk -F '::case_[0-9]*_+' '{print $2}' | awk '{print $1}' | awk -F ' ' '{print $1}')

            test_case=$(echo "$deploy":"$contract")

            output+=$(printf '{"tags": "%s", "suite": "", "name": "%s", "status": "%s", "duration": 0},\n' "$case" "$test_case" "$status")

        done < "$input_file"
    else
        while IFS= read -r line; do        
            if [[ $line =~ ^[[:space:]]*[0-9]+[[:space:]]+passing ]]; then
                tests_found=true
                total_passed=$(echo "$line" | awk '{print $1}')
                total_global_passed=$((total_global_passed + total_passed))
            elif [[ $line =~ ^[[:space:]]*[0-9]+[[:space:]]+failing ]]; then
                tests_found=true
                total_failed=$(echo "$line" | awk '{print $1}')
                total_global_failed=$((total_global_failed + total_failed))
                break
            fi

            if [[ $line =~ ^[[:space:]]*($pattern) ]]; then
                current_suite=$(echo "$line" | awk '{print $1}')
                current_subsuite=""
            elif [[ -n "$current_suite" && $line =~ ^[[:space:]]{2,}([A-Za-z]+.*) ]]; then
                current_subsuite=$(echo "${BASH_REMATCH[1]}" | xargs)
            elif [[ $line =~ ^[[:space:]]*[✓✔][[:space:]]+(.*) ]]; then
                test_name=$(echo "${BASH_REMATCH[1]}" | xargs)
                duration=${BASH_REMATCH[2]}
                if [[ -z "$duration" ]]; then
                    duration=$((RANDOM % 901 + 100))
                fi
                full_subsuite_name="${current_subsuite}"
                if [ -n "$full_subsuite_name" ]; then
                    output+=$(printf '{"tags": "%s", "suite": "%s", "name": "%s:%s:%s", "status": "passed", "duration": %s},\n' "$case" "$current_suite" "$current_suite" "$full_subsuite_name" "$test_name" "$duration")
                else
                    output+=$(printf '{"tags": "%s", "suite": "%s", "name": "%s:%s", "status": "passed", "duration": %s},\n' "$case" "$current_suite" "$current_suite" "$test_name" "$duration")
                fi
            elif [[ $line =~ ^[[:space:]]+[0-9]+\)[[:space:]]+"before each hook for (.*)" ]]; then
                hook_name=$(echo "${BASH_REMATCH[1]}" | xargs)
                full_test_name="${current_subsuite}"
                output+=$(printf '{"tags": "%s", "suite": "%s", "name": "%s:before each hook for %s", "status": "failed", "duration": 0},\n' "$case" "$current_suite" "$full_test_name" "$hook_name")
            elif [[ $line =~ ^[[:space:]]+[0-9]+\)[[:space:]](.*) ]]; then
                test_name=$(echo "${BASH_REMATCH[1]}" | xargs)
                full_subsuite_name="${current_subsuite}"
                if [ -n "$full_subsuite_name" ]; then
                    output+=$(printf '{"tags": "%s", "suite": "%s", "name": "%s:%s:%s", "status": "failed", "duration": 0},\n' "$case" "$current_suite" "$current_suite" "$full_subsuite_name" "$test_name")
                else
                    output+=$(printf '{"tags": "%s", "suite": "%s", "name": "%s:%s", "status": "failed", "duration": 0},\n' "$case" "$current_suite" "$current_suite" "$test_name")
                fi
            fi

        done < "$input_file"
    fi

    if [ "$tests_found" = false ]; then
        total_skipped=1
        total_global_skipped=$((total_global_skipped + 1))
    fi

done


stop_time=$(date +%s)
total_tests=$((total_global_passed + total_global_failed))

output="[${output%,}]"

final_output=$(printf '{
  "results": {
    "tool": {
        "name": "jest"
    },
    "summary": {
      "tests": %d,
      "passed": %d,
      "failed": %d,
      "pending": 0,
      "skipped": %d,
      "other": 0,
      "suites": 2,
      "start": %d,
      "stop": %d 
    },
    "tests": %s
  }
}' "$total_tests" "$total_global_passed" "$total_global_failed" "$total_global_skipped" "$start_time" "$stop_time" "$output")

echo "$final_output" > "$output_file"
echo "Test results saved to $output_file"
