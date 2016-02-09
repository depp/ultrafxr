using System;

namespace Moria.UltraFXR
{
	/// <summary>
	/// Attribute for continuous (floating-point) parameters.
	/// </summary>
	[System.AttributeUsage(System.AttributeTargets.Property)]
	public class ContinuousParamAttribute : ParamAttribute
	{
		public override object Default { get { return this.DefaultValue; } }

		/// <summary>
		/// Gets the parameter's initial value.
		/// </summary>
		public double DefaultValue { get; private set; }

		/// <summary>
		/// Gets the parameter's minimum value.
		/// </summary>
		public double Minimum { get; private set; }

		/// <summary>
		/// Gets the parameter's maximum value.
		/// </summary>
		public double Maximum { get; private set; }

		public ContinuousParamAttribute(int order, string name, double initVal, double minVal, double maxVal)
			: base(order, name)
		{
			this.DefaultValue = initVal;
			this.Minimum = minVal;
			this.Maximum = maxVal;
		}
	}
}
