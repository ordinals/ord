Ordinal Bounties
================

<table>
  <tr>
    <th>Type</th>
    <th>Supply</th>
    <th>Quota</th>
    <th>Bounty</th>
    <th>Address</th>
  </tr>
  <tr>
    <td>Epic</td>
    <td>3</td>
    <td>1</td>
    <td>1 BTC</td>
    <td>1PE7u4wbDP2RqfKN6geD1bG57v9Gj9FXm3</td>
  </tr>
  <tr>
    <td>Rare</td>
    <td>360</td>
    <td>10</td>
    <td>0.01 BTC</td>
    <td>145Z7PFHyVrwiMWwEcUmDgFbmUbQSU9aap</td>
  </tr>
  <tr>
    <td>Uncommon</td>
    <td>~730,000</td>
    <td>100</td>
    <td>0.001 BTC</td>
    <td>1Hyr94uypwWq5CQffaXHvwUMEyBPp3TUZH</td>
  </tr>
  <tr>
    <td>Elusive</td>
    <td>1</td>
    <td>1</td>
    <td>0.1 BTC</td>
    <td>13fq3gND5nHpYVYwnqEL6AuwxF5DBCDj1g</td>
  </tr>
  <tr>
    <td>Cursed</td>
    <td>4,999,999,999</td>
    <td>1</td>
    <td>0.1 BTC</td>
    <td>17m5rvMpi78zG8RUpCRd6NWWMJtWmu65kg</td>
  </tr>
  <tr>
    <td>Repeater</td>
    <td>137</td>
    <td>4</td>
    <td>0.025 BTC</td>
    <td>1KYfzhzGhYepGkWqzmqjRwStuYRkmTzR8r</td>
  </tr>
  <tr>
    <td>Round</td>
    <td>-</td>
    <td>15</td>
    <td>10<sup>trailing zeroes</sup> / 100,000,000 sats</td>
    <td>1QfaU6QdScFVyZVT2SW4qkQonkYAvBqQY</td>
  </tr>
  <tr>
    <td>Intelligible</td>
    <td>-</td>
    <td>100</td>
    <td>0.001 BTC</td>
    <td>1E4ZPUF9SQbW6WqLPVkW6q14uwr8CXBtHr</td>
  </tr>
</table>

Definitions
-----------

An *epic* satoshi is the first satoshi mined in the first block after a
halving. They are *exceedingly* rare. Three have been mined, and there is one
epic sat per 6,336,854 BTC in circulation.

A *rare* satoshi is the first satoshi mined in the first block after a
difficulty adjustment period. They are *very* rare. 361 have been mined, and
there is one epic sat per 52,423 BTC in circulation.

An *uncommon* satoshi is the first satoshi mined in every block. They are
moderately rare. More than 730,000 have been mined, and there is one rare sat
per 26 BTC in circulation.

There is only one *elusive* satoshi, that with ordinal number 623624999999999.
Why is left as an exercise to the reader.

All sats mined in block 124724 are *cursed*. Again, figuring out why is left as
an exercise to the reader.

A *repeater* satoshi's ordinal number consists of repititions a single digit,
for example, 777777777, or 3333. They are *extremely* rare. 137 have been
mined, or one per 138,138 BTC in circulation.

A *round* satoshi's ordinal number has many trailing zeros. The more, the
rarer. To reflect this, the bounty for round satoshies is variable.

An *intelligible* satoshi is one whose *name* is an word or sentence. The code
mapping ordinals to names is
[here](https://github.com/casey/ord/blob/79183599126485e73027fb34686cc0f29f56776d/src/ordinal.rs#L30).

Rules
-----

- Find an ordinal qualifying for a bounty.

- Make sure that the bounty has not already been claimed by verifying that no
  transactions have been sent to the bounty address.

- Send a transaction that sends the ordinal to the bounty address.

- Send a DM to [@rodarmor](https://twitter.com/rodarmor).

- The bounty will be sent back to the address that the ordinal was sent from.

Questions? Tweet them at [@rodarmor](https://twitter.com/rodarmor).

Hints
-----

- There are no ordinal wallets or transaction construction libraries. However,
  [ordinal theory](https://github.com/casey/ord/blob/master/bip.mediawiki) is
  extremely simple. A clever hacker should be able to write code to manipulate
  ordinals in no time.

- Extremely rare ordinals are nearly impossible to find. However, there are
  ways that an enterprising indivdual of limited means can still obtain rare
  sats.

- Round and intelligible sats are the easiest to find. You might want to start
  with them.

- Each of the bounty addresses is an [Opendime](https://opendime.com), so we
  don't accidentally lose any rare sats that are sent to us.

- For more information about ordinals, check out the [FAQ](/faq) for an
  overview, or the
  [BIP](https://github.com/casey/ord/blob/master/bip.mediawiki) for the
  technical details.

- Satoshi was the original developer of ordinal theory. However, he knew that
  others would consider it heretical and dangerous, so he hid his knowledge,
  and it was lost to the sands of time. This potent theory is only now being
  rediscovered. You can help by researching rare ordinals.

  The three epic ordinals mined so far are:

  <table>
    <tr>
      <th>Reward Era</th>
      <th>Block</th>
      <th>Ordinal</th>
      <th>Satpoint</th>
      <th>Address</th>
    </tr>
    <tr>
      <td>25 BTC</td>
      <td>210,000</td>
      <td>1,050,000,000,000,000</td>
      <td>817081e1e0574ca5352037769b99cdd826eeebec4d9f79e94d6bdefda4e6776e:0:35750468636</td>
      <td>1FC5pgkk2SQx5P9qiuAjL6x29bbZHBRzHN</td>
    </tr>
    <tr>
      <td>12.5 BTC</td>
      <td>420,000</td>
      <td>1,575,000,000,000,000</td>
      <td>eacb1ee7886bd5b8e6d72342d3dc9adb9e08167b858eb8299531a8aac635542a:81:3694792696</td>
      <td>1799w3Etgf6rPEs2yHKjNbzpM4yARHaKjY</td>
    </tr>
    <tr>
      <td>6.25 BTC</td>
      <td>630,000</td>
      <td>1,837,500,000,000,000</td>
      <td>9e6295ef5b69f6c87d704237dc0160fcc1b576b8eaff6fbb0b6124e5ca2a1adf:0:279869946431</td>
      <td>bc1qjysjfd9t9aspttpjqzv68k0ydpe7pvyd5vlyn37868473lell5tqkz456m</td>
    </tr>
  </table>

Good luck and godspeed!
