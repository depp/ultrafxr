using System;
using System.Collections.Generic;
using System.Reflection;

namespace Moria.UltraFXR
{
	/// <summary>
	/// Parameters for the BFXR-compatible synthesizer.
	/// </summary>
	public sealed class BfxrParameters
	{
		[EnumParam(0, "Wave Type", BfxrWave.Square)]
		public BfxrWave Wave { get; set; }

		[ContinuousParam(1, "Master Volume", 0.5, 0, 1)]
		public double MasterVolume { get; set; }

		[ContinuousParam(2, "Attack Time", 0, 0, 1)]
		public double AttackTime { get; set; }

		[ContinuousParam(3, "Sustain Time", 0.3, 0, 1)]
		public double SustainTime { get; set; }

		[ContinuousParam(4, "Punch", 0, 0, 1)]
		public double SustainPunch { get; set; }

		[ContinuousParam(5, "Decay Time", 0.4, 0, 1)]
		public double DecayTime { get; set; }

		[ContinuousParam(6, "Compression", 0.3, 0, 1)]
		public double CompressionAmount { get; set; }

		[ContinuousParam(7, "Frequency", 0.3, 0, 1)]
		public double StartFrequency { get; set; }

		[ContinuousParam(8, "Frequency Cutoff", 0, 0, 1)]
		public double MinFrequency { get; set; }

		[ContinuousParam(9, "Frequency Slide", 0, -1, 1)]
		public double Slide { get; set; }

		[ContinuousParam(10, "Delta Slide", 0, -1, 1)]
		public double DeltaSlide { get; set; }

		[ContinuousParam(11, "Vibrato Depth", 0, 0, 1)]
		public double VibratoDepth { get; set; }

		[ContinuousParam(12, "Vibrato Speed", 0, 0, 1)]
		public double VibratoSpeed { get; set; }

		[ContinuousParam(13, "Harmonics", 0, 0, 1)]
		public double Overtones { get; set; }

		[ContinuousParam(14, "Harmonics Falloff", 0, 0, 1)]
		public double OvertoneFalloff { get; set; }

		[ContinuousParam(15, "Pitch Jump Repeat Speed", 0, 0, 1)]
		public double ChangeRepeat { get; set; }

		[ContinuousParam(16, "Pitch Jump Amount 1", 0, -1, 1)]
		public double ChangeAmount { get; set; }

		[ContinuousParam(17, "Pitch Jump Onset 1", 0, 0, 1)]
		public double ChangeSpeed { get; set; }

		[ContinuousParam(18, "Pitch Jump Amount 2", 0, -1, 1)]
		public double ChangeAmount2 { get; set; }

		[ContinuousParam(19, "Pitch Jump Onset 2", 0, 0, 1)]
		public double ChangeSpeed2 { get; set; }

		[ContinuousParam(20, "Square Duty", 0, 0, 1)]
		public double SquareDuty { get; set; }

		[ContinuousParam(21, "Duty Sweep", 0, -1, 1)]
		public double DutySweep { get; set; }

		[ContinuousParam(22, "Repeat Speed", 0, 0, 1)]
		public double RepeatSpeed { get; set; }

		[ContinuousParam(23, "Flanger Offset", 0, -1, 1)]
		public double FlangerOffset { get; set; }

		[ContinuousParam(24, "Flanger Sweep", 0, -1, 1)]
		public double FlangerSweep { get; set; }

		[ContinuousParam(25, "Low-pass Filter Cutoff", 1, 0, 1)]
		public double LPFilterCutoff { get; set; }

		[ContinuousParam(26, "Low-pass Filter Cutoff Sweep", 0, -1, 1)]
		public double LPFilterCutoffSweep { get; set; }

		[ContinuousParam(27, "Low-pass Filter Resonance", 0, 0, 1)]
		public double LPFilterResonance { get; set; }

		[ContinuousParam(28, "High-pass Filter Cutoff", 0, 0, 1)]
		public double HPFilterCutoff { get; set; }

		[ContinuousParam(29, "High-pass Filter Cutoff Sweep", 0, -1, 1)]
		public double HPFilterCutoffSweep { get; set; }

		[ContinuousParam(30, "Bit Crush", 0, 0, 1)]
		public double BitCrush { get; set; }

		[ContinuousParam(31, "Bit Crush Sweep", 0, -1, 1)]
		public double BitCrushSweep { get; set; }

		/// <summary>
		/// Get the default BFXR parameters.
		/// </summary>
		/// <returns>BFXR parameters.</returns>
		public static BfxrParameters Default()
		{
			var p = new BfxrParameters();
			foreach (PropertyInfo prop in typeof(BfxrParameters).GetProperties(BindingFlags.Instance | BindingFlags.Public))
			{
				var attr = prop.GetCustomAttribute<ParamAttribute>();
				if (attr != null)
				{
					prop.SetValue(p, attr.Default);
				}
			}
			return p;
		}

		/// <summary>
		/// Deserialize a set of parameters from the BFXR tool.
		/// </summary>
		/// <param name="data">The BFXR parameterss, serialized as a string.</param>
		public static BfxrParameters Deserialize(string data)
		{
			string[] fieldText = data.Split(new char[]{ ',' });
			var p = new BfxrParameters();
			foreach (PropertyInfo prop in typeof(BfxrParameters).GetProperties(BindingFlags.Instance | BindingFlags.Public))
			{
				var attr = prop.GetCustomAttribute<ParamAttribute>();
				if (attr == null)
					continue;
				object value = attr.Default;
				string s = attr.Order < fieldText.Length ? fieldText[attr.Order] : null;
				if (s != null && s.Length > 0)
				{
					double v;
					if (!double.TryParse(s, out v))
					{
						Console.WriteLine("Warning: Cannot parse value for {0}: {1}", prop.Name, s);
					}
					else
					{
						var cattr = attr as ContinuousParamAttribute;
						if (cattr != null)
						{
							double nv = Math.Max(cattr.Minimum, Math.Min(cattr.Maximum, v));
							if (nv != v)
							{
								Console.WriteLine("Warning: Value {0} out of range for {1}, should be {2} .. {3}", v, prop.Name, cattr.Minimum, cattr.Maximum);
							}
							value = nv;
						}
						else
						{
							value = (BfxrWave)(Math.Max(0, Math.Min(8, (int)v)));
						}
					}
				}
				prop.SetValue(p, value);
			}
			return p;
		}

		/// <summary>
		/// Dump the parameters to the console.
		/// </summary>
		public void Dump()
		{
			var plist = new List<Tuple<int, string, string>>();
			foreach (PropertyInfo prop in typeof(BfxrParameters).GetProperties(BindingFlags.Instance | BindingFlags.Public))
			{
				var attr = prop.GetCustomAttribute<ParamAttribute>();
				if (attr != null)
				{
					plist.Add(Tuple.Create(attr.Order, prop.Name, prop.GetValue(this).ToString()));
				}
			}
			foreach (var prop in plist)
			{
				Console.WriteLine("    {0}: {1}", prop.Item2, prop.Item3);
			}
		}
	}
}
