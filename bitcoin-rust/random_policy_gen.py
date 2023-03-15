# usage: python3 random_policy_gen.py <sha256sum_of_your_code_submission_tarball>
# How to compute sha256sum of your submission tarball:
#   $ sha256sum <your_submission_tarball>

import sys
import json

arg1 = sys.argv[1]

# arg1 should be in the format of sha256sum in hex
assert len(arg1) == 64
assert all(c in "0123456789abcdef" for c in arg1)

# seed the random number generator using arg1
import random
random.seed(int(arg1, 16))

# read `seccomp_client.json`, `seccomp_nakamoto.json` and `seccomp_wallet.json` from `./bin_client/policies` in a loop
# for each policy, generate 2 random policies with one syscall in the whitelist dropped.
csv_lines = []
for policy in ["seccomp_client.json", "seccomp_nakamoto.json", "seccomp_wallet.json"]:
    with open("./bin_client/policies/" + policy, "r") as f:
        data = json.load(f)
    filter_list = data["main_thread"]["filter"]
    # randomly remove one entry from the filter_list
    for i in range(2):
        filter_list_copy = list(filter_list)
        poped_item = filter_list_copy.pop(random.randint(0, len(filter_list_copy) - 1))
        data_copy = json.loads(json.dumps(data))
        data["main_thread"]["filter"] = filter_list_copy
        policy_filename = policy[:-5] + "_DROP1_" + str(i) + ".json"
        with open("./bin_client/policies/" + policy_filename, "w") as f:
            json.dump(data, f, indent=2)
        csv_lines.append([policy_filename, poped_item, "Please_Replace_This_With_YES_or_NO"])

# write the csv file
import csv
with open("part_B_results.csv", "w") as f:
    csv_writer = csv.writer(f)
    csv_writer.writerow(["policy_file", "drop_entry", "is_crashing"])
    csv_writer.writerows(csv_lines)
