import React, { useEffect, useState } from "react";

import Chart from "chart.js";
import { tween } from "shifty";

import {
	actionPanel,
	displayAmount,
	home,
	historyScreen,
	spentChart,
	summary,
	withdrawScreen,
} from "./home.module.css";

const Home = () => {
	const [amount, setAmount] = useState(0);
	useEffect(() => {
		loadChart();

		tween({
			from: { x: 0 },
			to: { x: 569671200 },
			duration: 2500,
			easing: "easeInQuint",
			step: ({ x }) =>
				setAmount(
					new Intl.NumberFormat("pl-PL", {
						style: "currency",
						currency: "USD",
					}).format(x)
				),
		});
	}, []);
	return (
		<div className={home}>
			<div className={spentChart}>
				<canvas id="spentChart" height="100%"/>
			</div>
			<div className={summary}>
				<div className={withdrawScreen}>
					Savings
					<span className={displayAmount}>{amount}</span>
					<div className={actionPanel}>
						<button onClick={() => alert("We went bankrupt :(")}>
							Withdraw
						</button>
					</div>
				</div>
				<div className={historyScreen}>
					<ul>
						<li>
							<span>Payment</span>
							<span>- $1.23</span>
						</li>
						<li>
							<span>Earning</span>
							<span>+ $12.08</span>
						</li>
						<li>
							<span>Earning</span>
							<span>+ $3.5</span>
						</li>
						<li>
							<span>Payment</span>
							<span>- $42.6</span>
						</li>
						<li>
							<span>Payment</span>
							<span>- $136.8</span>
						</li>
						<li>
							<span>Payment</span>
							<span>- $12.4</span>
						</li>
					</ul>
				</div>
			</div>
		</div>
	);
};

export default Home;

Chart.defaults.global.defaultFontColor = '#FFF';
var config = {
	type: "line",
	data: {
		labels: ["January", "February", "March", "April", "May", "June", "July"],
		datasets: [
			{
				label: "Salary",
				borderColor: "#af5b5bff",
				backgroundColor: "#af5b5bff",
				data: [3, 5, 9, 2, 10, 4, 3, 7],
			},
			{
				label: "DeFi",
				borderColor: "#276fbfff",
				backgroundColor: "#276fbfff",
				data: [3, 5, 6, 5, 2, 7, 9, 12],
			},
			{
				label: "Airdrop lol",
				borderColor: "#f6f4f3ff",
				backgroundColor: "#f6f4f3ff",
				data: [8, 7, 5, 3, 8, 3, 8],
			},
		],
	},
	options: {
		responsive: true,
		title: {
			display: true,
			text: "Earnings",
		},
		tooltips: {
			mode: "index",
		},
		hover: {
			mode: "index",
		},
		scales: {
			x: {
				scaleLabel: {
					display: true,
					labelString: "Month",
				},
			},
			y: {
				stacked: true,
				scaleLabel: {
					display: true,
					labelString: "Value",
				},
			},
		},
	},
};

function loadChart() {
	let ctx = document.getElementById("spentChart").getContext("2d");
	window.myLine = new Chart(ctx, config);
}
